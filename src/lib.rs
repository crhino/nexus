// #![deny(missing_docs, dead_code)]

//! # Nexus
//!
//! A high performance networking library

#![cfg_attr(all(test, feature = "json_codec"), feature(custom_derive, plugin))]
#![cfg_attr(all(test, feature = "json_codec"), plugin(serde_macros))]

extern crate rotor;
extern crate netbuf;
extern crate void;
extern crate byteorder;
#[macro_use] extern crate log;

#[cfg(feature = "json_codec")] extern crate serde;
#[cfg(feature = "json_codec")] extern crate serde_json;

#[cfg(test)] extern crate ferrous;

pub mod traits;
pub mod future;
pub mod pipeline;
pub mod transport;
pub mod codec;

#[cfg(test)]
mod test_helpers;
