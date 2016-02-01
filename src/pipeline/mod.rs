//! # Pipelines
//!
//! Pipelines are the main unit of composition in Nexus.
//!
//! The ReadStage and WriteStage traits define a set of functions that will be called
//! when the related event occurs.

mod context;
pub use self::context::{Context};

mod chain;

mod pipeline;
pub use self::pipeline::{Pipeline};

use std::io;
use future::NexusFuture;

pub trait Stage {
    type Input;
    type Output;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context;
    fn closed<C>(&mut self, ctx: &mut C) where C: Context;
}

pub trait ReadStage: Stage {
    fn read<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> where C: Context;
}

pub trait WriteStage: Stage {
    fn write<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> where C: Context;
}
