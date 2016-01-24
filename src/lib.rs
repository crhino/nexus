// #![deny(missing_docs, dead_code)]

//! # Nexus
//!
//! A high performance networking library

extern crate rotor;
#[macro_use] extern crate log;

#[cfg(test)] extern crate ferrous;

mod future;
mod pipeline;
// mod reactor;
// pub use reactor::{Token, Reactor, ReactorError, ReactorConfig, Configurer};

// mod protocol;
// pub use protocol::Protocol;

// pub mod tcp;

#[cfg(test)]
mod test_helpers;
