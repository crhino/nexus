//! # TCP Reactor
//!
//! The TcpReactor and TcpProtocol are a further specification of Reactors that are scoped to TCP
//! protocols. The TcpReactor will accept all incoming connections and pass them to the protocol
//! with the new `on_connect` trait method provided by the `TcpProtocol` trait.
mod protocol;
pub use tcp::protocol::TcpProtocol;

mod tcp_reactor;
pub use tcp::tcp_reactor::{TcpReactor};
