// #![deny(missing_docs, dead_code)]

//! # Nexus
//!
//! A high performance networking library

extern crate rotor;
extern crate void;
#[macro_use] extern crate log;

#[cfg(test)] extern crate ferrous;

mod future;
mod pipeline;

#[cfg(test)]
mod test_helpers;
