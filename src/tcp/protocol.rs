use protocol::Protocol;
use reactor::{Token, Configurer, ReactorError};
use mio::tcp::{TcpStream, TcpListener};
use mio::{EventSet, Selector, PollOpt, Evented};
use std::io::{self, Error, ErrorKind};
use std::marker::PhantomData;

/// Trait used with a TCP reactor.
pub trait TcpProtocol: Protocol {
    /// Event called when a new tcp connections is received.
    ///
    /// The socket should be transformed into `Self::Socket` type and added to the reactor with the
    /// configurer.
    fn on_connect<C>(&mut self, configurer: &mut C, socket: TcpStream) where C: Configurer<Self::Socket>;
}

pub struct ReactorProtocol<P> {
    protocol: P,
}

impl<P> ReactorProtocol<P> {
    pub fn new(proto: P) -> ReactorProtocol<P> {
        ReactorProtocol{
            protocol: proto,
        }
    }
}

#[derive(Debug)]
pub enum TcpSocket<S> {
    Listener(TcpListener),
    Socket(S),
}

impl<S: Evented> Evented for TcpSocket<S> {
    fn register(&self, selector:
                &mut Selector,
                token: Token,
                interest: EventSet,
                opts: PollOpt) -> io::Result<()> {
        match *self {
            TcpSocket::Listener(ref l) => l.register(selector, token, interest, opts),
            TcpSocket::Socket(ref s) => s.register(selector, token, interest, opts),
        }
    }

    fn reregister(&self,
                  selector: &mut Selector,
                  token: Token,
                  interest: EventSet,
                  opts: PollOpt) -> io::Result<()> {
        match *self {
            TcpSocket::Listener(ref l) => l.reregister(selector, token, interest, opts),
            TcpSocket::Socket(ref s) => s.reregister(selector, token, interest, opts),
        }
    }

    fn deregister(&self, selector: &mut Selector) -> io::Result<()> {
        match *self {
            TcpSocket::Listener(ref l) => l.deregister(selector),
            TcpSocket::Socket(ref s) => s.deregister(selector),
        }
    }
}

struct TcpConfigurer<'a, S, C: Configurer<TcpSocket<S>> + 'a> {
    inner: &'a mut C,
    phantom: PhantomData<*const S>,
}

impl<'a, S, C: Configurer<TcpSocket<S>>> TcpConfigurer<'a, S, C> {
    fn new(inner: &'a mut C) -> TcpConfigurer<'a, S, C> {
        TcpConfigurer{
            inner: inner,
            phantom: PhantomData,
        }
    }
}

impl<'a, S, C: Configurer<TcpSocket<S>>> Configurer<S> for TcpConfigurer<'a, S, C> {
    fn add_socket(&mut self, socket: S, events: EventSet) {
        self.inner.add_socket(TcpSocket::Socket(socket), events)
    }

    fn add_socket_timeout(&mut self, socket: S, events: EventSet, timeout_ms: u64) {
        self.inner.add_socket_timeout(TcpSocket::Socket(socket), events, timeout_ms)
    }

    fn remove_socket(&mut self, token: Token) {
        self.inner.remove_socket(token)
    }

    fn update_socket(&mut self, token: Token, events: EventSet) {
        self.inner.update_socket(token, events)
    }

    fn update_socket_timeout(&mut self, token: Token, events: EventSet, timeout_ms: u64) {
        self.inner.update_socket_timeout(token, events, timeout_ms)
    }

    fn shutdown(&mut self, error: io::Error) {
        self.inner.shutdown(error)
    }
}

impl<P: TcpProtocol> TcpProtocol for ReactorProtocol<P> {
    fn on_connect<C>(&mut self,
                     configurer: &mut C,
                     socket: TcpStream) where C: Configurer<Self::Socket> {
        let mut c = TcpConfigurer::new(configurer);
        self.protocol.on_connect(&mut c, socket);
    }
}

impl<P: TcpProtocol> Protocol for ReactorProtocol<P> {
    type Socket = TcpSocket<P::Socket>;

    fn on_readable<C>(&mut self,
                      configurer: &mut C,
                      socket: &mut Self::Socket,
                      token: Token) where C: Configurer<Self::Socket> {
        let mut c = TcpConfigurer::new(configurer);
        match *socket {
            TcpSocket::Listener(ref mut l) => {
                loop {
                    let proto = &mut self.protocol;
                    match l.accept() {
                        Ok(Some((skt, _addr))) => {
                            proto.on_connect(&mut c, skt);
                        },
                        Ok(None) => break,
                        Err(e) => {
                            error!("error accepting connections: {:?}", e);
                            c.shutdown(e);
                            return
                        },
                    }
                }
            },
            TcpSocket::Socket(ref mut s) => {
                self.protocol.on_readable(&mut c, s, token)
            },
        }
    }

    fn on_writable<C>(&mut self,
                      configurer: &mut C,
                      socket: &mut Self::Socket,
                      token: Token) where C: Configurer<Self::Socket> {
        match *socket {
            TcpSocket::Listener(_) => {
                error!("received writable event for listener");
            },
            TcpSocket::Socket(ref mut s) => {
                let mut c = TcpConfigurer::new(configurer);
                self.protocol.on_writable(&mut c, s, token)
            },
        }
    }

    fn on_timeout<C>(&mut self,
                     configurer: &mut C,
                     socket: &mut Self::Socket,
                     token: Token) where C: Configurer<Self::Socket> {
        match *socket {
            TcpSocket::Listener(_) => {
                error!("received timeout event for listener");
            },
            TcpSocket::Socket(ref mut s) => {
                let mut c = TcpConfigurer::new(configurer);
                self.protocol.on_timeout(&mut c, s, token)
            },
        }
    }

    fn on_disconnect<C>(&mut self,
                        configurer: &mut C,
                        socket: &mut Self::Socket,
                        token: Token) where C: Configurer<Self::Socket> {
        match *socket {
            TcpSocket::Listener(_) => {
                error!("received disconnect event for listener");
                let err = Error::new(ErrorKind::Other, "listener disconnected");
                configurer.shutdown(err);
            },
            TcpSocket::Socket(ref mut s) => {
                let mut c = TcpConfigurer::new(configurer);
                self.protocol.on_disconnect(&mut c, s, token)
            },
        }
    }

    fn on_socket_error<C>(&mut self,
                          configurer: &mut C,
                          socket: &mut Self::Socket,
                          token: Token) where C: Configurer<Self::Socket> {
        match *socket {
            TcpSocket::Listener(_) => {
                error!("received socket error event for listener");
                let err = Error::new(ErrorKind::Other, "listener socket error");
                configurer.shutdown(err);
            },
            TcpSocket::Socket(ref mut s) => {
                let mut c = TcpConfigurer::new(configurer);
                self.protocol.on_socket_error(&mut c, s, token)
            },
        }
    }

    fn on_event_loop_error(&mut self, error: ReactorError<Self::Socket>) {
        match error {
            ReactorError::IoError(err, s) => {
                match s {
                    TcpSocket::Listener(_) => {
                        error!("received event loop error for listener: {:?}", err);
                        // TODO: Make this better
                        panic!(err);
                    },
                    TcpSocket::Socket(skt) => {
                        let err = ReactorError::IoError(err, skt);
                        self.protocol.on_event_loop_error(err)
                    },
                }
            },
            ReactorError::NoSocketFound(t) => {
                self.protocol.on_event_loop_error(ReactorError::NoSocketFound(t))
            },
            ReactorError::TimerError => {
                self.protocol.on_event_loop_error(ReactorError::TimerError)
            },
        }
    }

    fn tick<C>(&mut self, configurer: &mut C) where C: Configurer<Self::Socket> {
        let mut c = TcpConfigurer::new(configurer);
        self.protocol.tick(&mut c)
    }
}

#[cfg(test)]
mod tests {
    use mio::tcp::{TcpListener, TcpStream};
    use mio::{Token};
    use test_helpers::{FakeTcpProtocol};
    use protocol::Protocol;
    use std::thread;
    use super::{ReactorProtocol, TcpSocket};
    use reactor::configurer::{ProtocolConfigurer};

    #[test]
    fn test_adding_new_connections() {
        let l = TcpListener::bind(&"127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = l.local_addr().unwrap();
        let proto = FakeTcpProtocol::new();
        let mut reactor_proto = ReactorProtocol::new(proto.clone());
        let mut configurer = ProtocolConfigurer::new();

        let handle = thread::spawn(move || {
            let _stream1 = TcpStream::connect(&addr).unwrap();
            let _stream2 = TcpStream::connect(&addr).unwrap();
        });

        handle.join().unwrap();
        reactor_proto.on_readable(&mut configurer, &mut TcpSocket::Listener(l), Token(0));

        assert_eq!(proto.connect_count(), 2);
    }
}
