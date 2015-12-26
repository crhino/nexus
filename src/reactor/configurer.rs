use mio::{EventSet, EventLoop};

use protocol::Protocol;
use reactor::{Reactor, ReactorError, ReactorHandler, Token};

pub struct ProtocolConfigurer<P: Protocol> {
    pub additions: Vec<(P::Socket, EventSet, Option<u64>)>,
    pub removals: Vec<Token>,
    pub updates: Vec<(Token, EventSet, Option<u64>)>,
}

impl<P: Protocol> ProtocolConfigurer<P> {
    pub fn new() -> ProtocolConfigurer<P> {
        ProtocolConfigurer{
            additions: Vec::new(),
            removals: Vec::new(),
            updates: Vec::new(),
        }
    }

    pub fn update_event_loop(self,
                             event_loop: &mut EventLoop<ReactorHandler<P>>,
                             handler: &mut ReactorHandler<P>) {
        for (s, evt, timeout) in self.additions.into_iter() {
            match handler.add_socket(event_loop, s, evt, timeout) {
                Ok(_) => {},
                Err(e) => handler.event_loop_error(e),
            }
        }

        for (s, evt, timeout) in self.updates.into_iter() {
            match handler.update_socket(event_loop, s, evt, timeout) {
                Ok(_) => {},
                Err(e) => handler.event_loop_error(e),
            }
        }

        for s in self.removals.into_iter() {
            match handler.remove_socket(event_loop, s) {
                Ok(_) => {},
                Err(e) => handler.event_loop_error(e),
            }
        }
    }
}

/// Trait to allow a Protocol implementation to configure the Reactor's sockets.
pub trait Configurer<P: Protocol> {
    /// Add socket to the Reactor.
    fn add_socket(&mut self, socket: P::Socket, events: EventSet);

    /// Add socket to the Reactor with a timeout.
    fn add_socket_timeout(&mut self, socket: P::Socket, events: EventSet, timeout_ms: u64);

    /// Remove the socket associated to the token from the Reactor.
    fn remove_socket(&mut self, token: Token);

    /// Update the socket associated to the token from the Reactor.
    fn update_socket(&mut self, token: Token, events: EventSet);

    /// Update the socket associated to the token from the Reactor with a timeout.
    fn update_socket_timeout(&mut self, token: Token, events: EventSet, timeout_ms: u64);
}

impl<P: Protocol> Configurer<P> for ProtocolConfigurer<P> {
    fn add_socket(&mut self, socket: P::Socket, events: EventSet) {
        self.additions.push((socket, events, None));
    }

    fn add_socket_timeout(&mut self, socket: P::Socket, events: EventSet, timeout_ms: u64) {
        self.additions.push((socket, events, Some(timeout_ms)));
    }

    fn remove_socket(&mut self, socket: Token) {
        self.removals.push(socket);
    }

    fn update_socket(&mut self, socket: Token, events: EventSet) {
        self.updates.push((socket, events, None));
    }

    fn update_socket_timeout(&mut self, token: Token, events: EventSet, timeout_ms: u64) {
        self.updates.push((token, events, Some(timeout_ms)));
    }
}

impl<P: Protocol> Reactor<P> {
    /// Add socket to the Reactor.
    pub fn add_socket(&mut self, socket: P::Socket, events: EventSet)
        -> Result<Token, ReactorError<P::Socket>> {
            let handler = &mut self.1;
            let event_loop = &mut self.0;
            handler.add_socket(event_loop, socket, events, None)
        }

    /// Add socket to the Reactor with a timeout.
    pub fn add_socket_timeout(&mut self,
                              socket: P::Socket,
                              events: EventSet,
                              timeout_ms: u64)
        -> Result<Token, ReactorError<P::Socket>> {
            let handler = &mut self.1;
            let event_loop = &mut self.0;
            handler.add_socket(event_loop, socket, events, Some(timeout_ms))
        }

    /// Remove the socket associated to the token from the Reactor.
    pub fn remove_socket(&mut self, token: Token)
        -> Result<P::Socket, ReactorError<P::Socket>> {
            let handler = &mut self.1;
            let event_loop = &mut self.0;
            handler.remove_socket(event_loop, token)
        }

    /// Update the socket associated to the token from the Reactor.
    pub fn update_socket(&mut self, token: Token, events: EventSet)
        -> Result<(), ReactorError<P::Socket>> {
            let handler = &mut self.1;
            let event_loop = &mut self.0;
            handler.update_socket(event_loop, token, events, None)
        }

    /// Update the socket associated to the token from the Reactor with a timeout.
    pub fn update_socket_timeout(&mut self, token: Token, events: EventSet, timeout_ms: u64)
        -> Result<(), ReactorError<P::Socket>> {
            let handler = &mut self.1;
            let event_loop = &mut self.0;
            handler.update_socket(event_loop, token, events, Some(timeout_ms))
        }
}
