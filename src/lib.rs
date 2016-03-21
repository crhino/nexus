// #![deny(missing_docs, dead_code)]

//! # Nexus
//!
//! A high performance networking library

extern crate rotor;
extern crate netbuf;
extern crate void;
#[macro_use] extern crate log;

#[cfg(test)] extern crate ferrous;

pub mod traits;
pub mod future;
pub mod pipeline;

#[cfg(test)]
mod test_helpers;
