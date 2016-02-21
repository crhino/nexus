//! # Pipelines
//!
//! Pipelines are the main unit of composition in Nexus.
//!
//! The ReadStage and WriteStage traits define a set of functions that will be called
//! when the related event occurs.

pub mod context;
pub use self::context::{Context};

mod chain;

mod pipeline;
pub use self::pipeline::{Pipeline};

use std::io;
use std::marker::PhantomData;
use future::{Promise};

pub trait Stage<S> {
    type ReadInput;
    type ReadOutput;
    type WriteInput;
    type WriteOutput;

    fn connected<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S, Write=Self::WriteOutput>;
    fn closed<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S>;
    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput) -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput>;
    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput, promise: Promise<()>)
        -> Option<(Self::WriteOutput, Promise<()>)>
            where C: Context<Socket=S>;
}

pub trait ReadStage<S> { type Input;
    type Output;

    fn connected<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S>;
    fn closed<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S>;
    fn read<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output>
            where C: Context<Socket=S>;
}

pub struct ReadOnlyStage<R, W> {
    read_stage: R,
    write: PhantomData<* const W>,
}

impl<R, W> ReadOnlyStage<R, W> {
    pub fn new(read_stage: R) -> ReadOnlyStage<R, W> {
        ReadOnlyStage {
            read_stage: read_stage,
            write: PhantomData,
        }
    }
}

impl<R: ReadStage<S>, S, W> Stage<S> for ReadOnlyStage<R, W> {
    type ReadInput = R::Input;
    type ReadOutput = R::Output;
    type WriteInput = W;
    type WriteOutput = W;

    fn connected<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S, Write=Self::WriteOutput> {
        self.read_stage.connected(ctx)
    }

    fn closed<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S> {
        self.read_stage.closed(ctx)
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput)
        -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput> {
        self.read_stage.read(ctx, input)
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput, promise: Promise<()>)
        -> Option<(Self::WriteOutput, Promise<()>)>
            where C: Context<Socket=S> {
        Some((input, promise))
    }
}

pub trait WriteStage<S> {
    type Input;
    type Output;

    fn connected<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S, Write=Self::Output>;
    fn closed<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S>;
    fn write<C>(&mut self, ctx: &mut C, input: Self::Input, promise: Promise<()>)
        -> Option<(Self::Output, Promise<()>)>
            where C: Context<Socket=S>;
}

pub struct WriteOnlyStage<R, W> {
    write_stage: W,
    read: PhantomData<* const R>,
}

impl<R, W> WriteOnlyStage<R, W> {
    pub fn new(write_stage: W) -> WriteOnlyStage<R, W> {
        WriteOnlyStage {
            write_stage: write_stage,
            read: PhantomData,
        }
    }
}

impl<S, R, W: WriteStage<S>> Stage<S> for WriteOnlyStage<R, W> {
    type ReadInput = R;
    type ReadOutput = R;
    type WriteInput = W::Input;
    type WriteOutput = W::Output;

    fn connected<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S, Write=Self::WriteOutput> {
        self.write_stage.connected(ctx)
    }

    fn closed<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S> {
        self.write_stage.closed(ctx)
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput)
        -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput> {
        Some(input)
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput, promise: Promise<()>)
        -> Option<(Self::WriteOutput, Promise<()>)>
            where C: Context<Socket=S> {
        self.write_stage.write(ctx, input, promise)
    }
}
