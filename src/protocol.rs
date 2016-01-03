use mio::{Evented};
use reactor::{ReactorError, Configurer, Token};

/// A trait representing a network Protocol
pub trait Protocol {
    /// Socket type for the Protocol
    type Socket: Evented;

    /// Called right before starting the Reactor.
    fn on_start<C>(&mut self, configurer: &mut C) where C: Configurer<Self::Socket>;

    /// Called when the socket is readable.
    fn on_readable<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket>;

    /// Called when the socket is writable.
    fn on_writable<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket>;

    /// Called when the timeout for a socket has been reached without any events.
    fn on_timeout<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket>;

    /// Called when the socket has been disconnected.
    fn on_disconnect<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket>;

    /// Called when an error on the socket happens.
    fn on_socket_error<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket>;

    /// Called when an error registering the socket with the event loop happens.
    fn on_event_loop_error<C>(&mut self, configurer: &mut C, error: ReactorError<Self::Socket>) where C: Configurer<Self::Socket>;

    /// Called at the end of of the run loop.
    fn tick<C>(&mut self, configurer: &mut C) where C: Configurer<Self::Socket>;
}
