//! TCP Reactor
//!
//! The TcpReactor and TcpProtocol are a further specification of Reactors that are scoped to TCP
//! protocols. The TcpReactor will accept all incoming connections and pass them to the protocol
//! with the new `on_connect` trait method provided by the `TcpProtocol` trait.
//!
//! # Examples
//!
//! ```no_run
//! extern crate mio;
//! extern crate nexus;
//! # use nexus::{Protocol};
//! # use nexus::tcp::TcpProtocol;
//! # use mio::tcp::TcpStream;
//! # use nexus::{Token, Configurer, ReactorError};
//! # struct FakeTcpProtocol;
//! # impl Protocol for FakeTcpProtocol {
//! #    type Socket = TcpStream;
//! #
//! #    fn on_readable(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {}
//! #
//! #    fn on_writable(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {}
//! #
//! #    fn on_timeout(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {}
//! #
//! #    fn on_disconnect(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {}
//! #
//! #    fn on_socket_error(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {}
//! #
//! #    fn on_event_loop_error(&mut self, error: ReactorError<Self::Socket>) {}
//! #
//! #    fn tick(&mut self, _configurer: &mut Configurer<Self::Socket>) {}
//! # }
//! #
//! # impl TcpProtocol for FakeTcpProtocol {
//! #    fn on_connect(&mut self, configurer: &mut Configurer<Self::Socket>, socket: TcpStream) {}
//! # }
//!
//! use nexus::tcp::TcpReactor;
//! use mio::tcp::TcpListener;
//!
//! fn main() {
//!     let listener = TcpListener::bind(&"127.0.0.1:0".parse().unwrap()).unwrap();
//!     let protocol = FakeTcpProtocol;
//!     let mut reactor = TcpReactor::new(protocol, listener).unwrap();
//!     reactor.run().unwrap();
//! }
//! ```
mod protocol;
pub use tcp::protocol::TcpProtocol;

mod tcp_reactor;
pub use tcp::tcp_reactor::{TcpReactor};
