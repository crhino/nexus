use pipeline::{Context, Stage, WriteStage, ReadStage};
use future::{Promise};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct FakePassthroughStage<R, W> {
    r: PhantomData<*const R>,
    w: PhantomData<*const W>,
}

impl<R, W> FakePassthroughStage<R, W> {
    pub fn new() -> FakePassthroughStage<R, W> {
        FakePassthroughStage {
            r: PhantomData,
            w: PhantomData,
        }
    }
}

impl<S, R, W> Stage<S> for FakePassthroughStage<R, W> {
    type ReadInput = R;
    type ReadOutput = R;
    type WriteInput = W;
    type WriteOutput = W;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput)
        -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput> {
        Some(input)
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput, promise: Promise<()>)
        -> Option<(Self::WriteOutput, Promise<()>)>
            where C: Context<Socket=S> {
        Some((input, promise))
    }
}
