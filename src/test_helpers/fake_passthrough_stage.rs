use pipeline::{Context, Stage, WriteStage, ReadStage};
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

impl<'a, R: 'a, W: 'a> Stage<'a> for FakePassthroughStage<R, W> {
    type ReadInput = R;
    type ReadOutput = R;
    type WriteInput = W;
    type WriteOutput = W;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput) -> Option<Self::ReadOutput> where C: Context {
        Some(input)
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput) -> Option<Self::WriteOutput> where C: Context {
        Some(input)
    }
}
