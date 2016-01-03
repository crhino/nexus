use protocol::{Protocol};
use reactor::configurer::{ProtocolConfigurer};
use reactor::{ReactorError, Configurer, ShutdownHandle, SLAB_GROW_SIZE};
use mio::{Timeout, EventLoop, EventSet, Token, PollOpt, Handler};
use mio::util::Slab;
use std::io;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};

enum UpdateError {
    NoSocketFound(Token),
    IoError(io::Error)
}

pub struct ReactorHandler<P: Protocol> {
    protocol: P,
    slab: Slab<P::Socket>,
    timeouts: HashMap<Token, Timeout>,
    shutdown: Arc<AtomicBool>,
    proto_error: Option<io::Error>,
}

impl<P: Protocol> ReactorHandler<P> {
    pub fn new(proto: P, shutdown: Arc<AtomicBool>) -> ReactorHandler<P> {
        //TODO: Figure out what to actually start at
        let skt_slab = Slab::new_starting_at(Token(100), SLAB_GROW_SIZE);
        let timeout_map = HashMap::with_capacity(SLAB_GROW_SIZE);
        ReactorHandler{
            protocol: proto,
            slab: skt_slab,
            timeouts: timeout_map,
            shutdown: shutdown,
            proto_error: None,
        }
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle(self.shutdown.clone())
    }

    pub fn set_protocol_error(&mut self, err: io::Error) {
        if self.proto_error.is_none() {
            println!("Setting proto error");
            self.proto_error = Some(err);
        }
    }

    pub fn protocol_error(&mut self) -> Option<io::Error> {
        self.proto_error.take()
    }

    pub fn init_protocol(&mut self, event_loop: &mut EventLoop<Self>) -> io::Result<()> {
        let mut configurer = ProtocolConfigurer::new();
        self.protocol.on_start(&mut configurer);
        configurer.update_event_loop(event_loop, self);
        match self.protocol_error() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    fn slab<'a>(&'a mut self) -> &'a mut Slab<P::Socket> {
        &mut self.slab
    }

    fn call_protocol<C: Configurer<P::Socket>>(&mut self,
                                    configurer: &mut C,
                                    token: Token,
                                    events: EventSet) {
        let socket = &mut self.slab[token];
        if events.is_error() {
            self.protocol.on_socket_error(configurer, socket, token);
        }

        if events.is_hup() {
            self.protocol.on_disconnect(configurer, socket, token);
        }

        if events.is_readable() {
            self.protocol.on_readable(configurer, socket, token);
        }

        if events.is_writable() {
            self.protocol.on_writable(configurer, socket, token);
        }
    }

    pub fn event_loop_error(&mut self, event_loop: &mut EventLoop<Self>, err: ReactorError<P::Socket>) {
        let mut configurer = ProtocolConfigurer::new();
        self.protocol.on_event_loop_error(&mut configurer, err);
        configurer.update_event_loop(event_loop, self);
    }

    fn add_timeout(&mut self, event_loop: &mut EventLoop<Self>, token: Token, time: u64)
        -> Result<Token, ReactorError<P::Socket>> {
            match event_loop.timeout_ms(token, time) {
                Ok(timer) => { Ok(self.timeouts.insert(token, timer)) },
                Err(e) => {
                    warn!("failed to add timeout for token {:?}: {:?}", token, e);
                    Err(ReactorError::TimerError)
                }
            }.and_then(|opt_t| {
                // Clear timeout if old timer was found at token's position
                self.clear_timeout(event_loop, opt_t);
                Ok(token)
            })
        }


    fn clear_timeout(&self, event_loop: &mut EventLoop<Self>, timeout: Option<Timeout>) {
        match timeout {
            Some(t) => {
                event_loop.clear_timeout(t);
            },
            None => {},
        }
    }

    fn add_to_slab_and_register(&mut self,
                                event_loop: &mut EventLoop<Self>,
                                socket: P::Socket,
                                events: EventSet)
        -> Result<Token, ReactorError<P::Socket>> {
            let slab = self.slab();
            match slab.insert(socket) {
                Ok(token) => {
                    debug!("Inserted token {:?} into slab", token);
                    Ok(token)
                },
                Err(socket) => {
                    slab.grow(SLAB_GROW_SIZE);
                    slab.insert(socket)
                        .or_else(|_s| {
                            panic!("Could not insert socket but just grew slab")
                        })
                },
            }.and_then(|token| {
                let res = {
                    let conn = &mut slab[token];
                    event_loop.register(conn, token, events, PollOpt::edge())
                };
                res.and_then(|_| {
                    Ok(token)
                }).or_else(|e| {
                    match slab.remove(token) {
                        Some(s) => Err(ReactorError::IoError(e, s)),
                        None => Err(ReactorError::NoSocketFound(token)),
                    }
                })
            })
        }

    pub fn add_socket(&mut self,
                      event_loop: &mut EventLoop<Self>,
                      socket: P::Socket,
                      events: EventSet,
                      timeout: Option<u64>) -> Result<Token, ReactorError<P::Socket>> {
        self.add_to_slab_and_register(event_loop, socket, events)
            .and_then(|token| {
                match timeout {
                    Some(time_ms) => self.add_timeout(event_loop, token, time_ms),
                    None => Ok(token),
                }
            })
    }

    pub fn remove_socket(&mut self,
                         event_loop: &mut EventLoop<Self>,
                         token: Token) -> Result<P::Socket, ReactorError<P::Socket>> {
        let slab = self.slab();
        match slab.remove(token) {
            Some(s) => Ok(s),
            None => {
                warn!("token {:?} could not be found in slab", token);
                Err(ReactorError::NoSocketFound(token))
            },
        }.and_then(|socket| {
            match event_loop.deregister(&socket) {
                Ok(_) => Ok(socket),
                Err(e) => Err(ReactorError::IoError(e, socket))
            }
        })
    }

    fn reregister_socket(&mut self,
                         event_loop: &mut EventLoop<Self>,
                         token: Token,
                         events: EventSet) -> Result<(), UpdateError> {
        let slab = self.slab();
        match slab.get(token) {
            Some(s) => Ok(s),
            None => {
                warn!("token {:?} could not be found in slab", token);
                Err(UpdateError::NoSocketFound(token))
            },
        }.and_then(|socket| {
            event_loop.reregister(socket, token, events, PollOpt::edge())
            .map_err(UpdateError::IoError)
        })
    }

    fn translate_error_and_remove(&mut self, token: Token, err: UpdateError)
        -> Result<(), ReactorError<P::Socket>> {
            // Use UpdateError here because we need to drop the borrow
            // of the slab above in order to remove the socket from the
            // slab on error registering. Don't want to add another
            // error enum to ReactorError for an IoError without a socket
            let slab = self.slab();
            match err {
                UpdateError::NoSocketFound(token) => Err(ReactorError::NoSocketFound(token)),
                UpdateError::IoError(e) => {
                    match slab.remove(token) {
                        Some(s) => Ok(s),
                        None => {
                            warn!("token {:?} could not be found in slab", token);
                            Err(ReactorError::NoSocketFound(token))
                        },
                    }.and_then(|skt| {
                        Err(ReactorError::IoError(e, skt))
                    })
                }
            }
        }

    pub fn update_socket(&mut self,
                         event_loop: &mut EventLoop<Self>,
                         token: Token,
                         events: EventSet,
                         timeout: Option<u64>) -> Result<(), ReactorError<P::Socket>> {
        self.reregister_socket(event_loop, token, events)
            .or_else(|e| {
                self.translate_error_and_remove(token, e)
            }).and_then(|_| {
                match timeout {
                    Some(time_ms) => {
                        self.add_timeout(event_loop, token, time_ms).and_then(|_| { Ok(()) })
                    },
                    None => Ok(()),
                }
            })
    }

    fn timeout<C>(&mut self,
               configurer: &mut C,
               timeout: Token) where C: Configurer<P::Socket> {
        let socket = &mut self.slab[timeout];
        self.timeouts.remove(&timeout);
        self.protocol.on_timeout(configurer, socket, timeout);
    }
}

impl<P: Protocol> Handler for ReactorHandler<P> {
    type Timeout = Token;
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        let mut configurer = ProtocolConfigurer::new();
        self.call_protocol(&mut configurer, token, events);
        configurer.update_event_loop(event_loop, self);
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Self::Timeout) {
        let mut configurer = ProtocolConfigurer::new();
        self.timeout(&mut configurer, timeout);
        configurer.update_event_loop(event_loop, self);
    }

    fn tick(&mut self, event_loop: &mut EventLoop<Self>) {
        let mut configurer = ProtocolConfigurer::new();
        self.protocol.tick(&mut configurer);
        configurer.update_event_loop(event_loop, self);

        // Check shutdown at end of tick
        if self.shutdown.load(Ordering::SeqCst) {
            info!("shutting down event loop");
            event_loop.shutdown();
        }
    }
}
