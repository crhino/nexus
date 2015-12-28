use mio::{Evented};
use reactor::{ReactorError, Configurer, Token};

/// A trait representing a network Protocol
pub trait Protocol {
    /// Socket type for the Protocol
    type Socket: Evented;

    /// Called when the socket is readable.
    fn on_readable(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token);

    /// Called when the socket is writable.
    fn on_writable(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token);

    /// Called when the timeout for a socket has been reached without any events.
    fn on_timeout(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token);

    /// Called when the socket has been disconnected.
    fn on_disconnect(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token);

    /// Called when an error on the socket happens.
    fn on_socket_error(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token);

    /// Called when an error registering the socket with the event loop happens.
    fn on_event_loop_error(&mut self, error: ReactorError<Self::Socket>);

    /// Called at the end of of the run loop.
    fn tick(&mut self, configurer: &mut Configurer<Self::Socket>);
}
