#![deny(missing_docs, dead_code)]

//! # Nexus
//!
//! A high performance networking library

extern crate mio;
#[macro_use] extern crate log;

mod reactor;
pub use reactor::{Reactor, ReactorError, ReactorConfig, Configurer};

mod protocol;
pub use protocol::Protocol;

// pub mod tcp;

#[cfg(test)]
mod test_helpers;
