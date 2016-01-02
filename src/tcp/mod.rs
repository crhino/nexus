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
//! #   fn on_readable<C>(&mut self, _configurer: &mut C, _socket: &mut Self::Socket, _token: Token) where C: Configurer<Self::Socket> {}
//! #
//! #   fn on_writable<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket> {}
//! #
//! #   fn on_timeout<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket> {}
//! #
//! #   fn on_disconnect<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket> {}
//! #
//! #   fn on_socket_error<C>(&mut self, configurer: &mut C, socket: &mut Self::Socket, token: Token) where C: Configurer<Self::Socket> {}
//! #
//! #   fn on_event_loop_error(&mut self, error: ReactorError<Self::Socket>) {
//! #       panic!("fake tcp event loop error: {:?}", error)
//! #   }

//! #   fn tick<C>(&mut self, configurer: &mut C) where C: Configurer<Self::Socket> {}
//! # }
//! #
//! # impl TcpProtocol for FakeTcpProtocol {
//! #    fn on_connect<C>(&mut self, configurer: &mut C, socket: TcpStream) where C: Configurer<Self::Socket> {}
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
