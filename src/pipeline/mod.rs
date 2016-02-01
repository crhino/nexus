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

pub trait Stage<C: Context> {
    type Input;
    type Output;

    fn connected(&mut self, ctx: &mut C);
    fn closed(&mut self, ctx: &mut C);
}

pub trait ReadStage<C>: Stage<C> {
    fn read(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output>;
}

pub trait WriteStage<C>: Stage<C> {
    fn write<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output>;
}
