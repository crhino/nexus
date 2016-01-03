use mio::{EventSet, EventLoop, Evented};

use protocol::Protocol;
use reactor::{Reactor, ReactorError, ReactorHandler, Token};
use std::io;

pub struct ProtocolConfigurer<S> {
    additions: Vec<(S, EventSet, Option<u64>)>,
    removals: Vec<Token>,
    updates: Vec<(Token, EventSet, Option<u64>)>,
    error: Option<io::Error>,
}

impl<S: Evented> ProtocolConfigurer<S> {
    pub fn new() -> ProtocolConfigurer<S> {
        ProtocolConfigurer{
            additions: Vec::new(),
            removals: Vec::new(),
            updates: Vec::new(),
            error: None,
        }
    }

    pub fn update_event_loop<P: Protocol<Socket=S>>(mut self,
                             event_loop: &mut EventLoop<ReactorHandler<P>>,
                             handler: &mut ReactorHandler<P>) {
        match self.error.take() {
            Some(err) => {
                handler.set_protocol_error(err);
                handler.shutdown_handle().shutdown();
                return;
            },
            None => {},
        }

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
pub trait Configurer<S> {
    /// Add socket to the Reactor.
    fn add_socket(&mut self, socket: S, events: EventSet);

    /// Add socket to the Reactor with a timeout.
    fn add_socket_timeout(&mut self, socket: S, events: EventSet, timeout_ms: u64);

    /// Remove the socket associated to the token from the Reactor.
    fn remove_socket(&mut self, token: Token);

    /// Update the socket associated to the token from the Reactor.
    fn update_socket(&mut self, token: Token, events: EventSet);

    /// Update the socket associated to the token from the Reactor with a timeout.
    fn update_socket_timeout(&mut self, token: Token, events: EventSet, timeout_ms: u64);

    /// Signal reactor to shutdown at the end of the current iteration.
    ///
    /// The error passed in will be returned from the Reactor's run function. If this function
    /// is called more than once in a single iteration, the first error will be returned.
    fn shutdown(&mut self, error: io::Error);
}

impl<S> Configurer<S> for ProtocolConfigurer<S> {
    fn add_socket(&mut self, socket: S, events: EventSet) {
        self.additions.push((socket, events, None));
    }

    fn add_socket_timeout(&mut self, socket: S, events: EventSet, timeout_ms: u64) {
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

    fn shutdown(&mut self, error: io::Error) {
        if self.error.is_none() {
            self.error = Some(error);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use reactor::{Reactor};
    use protocol::Protocol;
    use std::io::{self, ErrorKind};
    use std::error::Error;
    use test_helpers::{FakeProtocol};

    #[test]
    fn test_configurer_shutdown() {
        let mut configurer: ProtocolConfigurer<<FakeProtocol as Protocol>::Socket>
            = ProtocolConfigurer::new();
        let proto = FakeProtocol::new();
        let Reactor(ref mut event_loop, ref mut handler) = Reactor::new(proto.clone()).unwrap();

        let err = io::Error::new(ErrorKind::Other, "oh no!");
        configurer.shutdown(err);
        configurer.update_event_loop(event_loop, handler);

        let err = handler.protocol_error();
        assert!(err.is_some());
        let err = err.unwrap();
        assert_eq!(err.kind(), ErrorKind::Other);
        assert_eq!(err.description(), "oh no!");
    }
}
