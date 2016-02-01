use pipeline::{Context, Stage, WriteStage, ReadStage};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct FakePassthroughStage<I> {
    input: PhantomData<*const I>,
}

impl<I> FakePassthroughStage<I> {
    pub fn new() -> FakePassthroughStage<I> {
        FakePassthroughStage {
            input: PhantomData,
        }
    }
}

impl<I> Stage for FakePassthroughStage<I> {
    type Input = I;
    type Output = I;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
    }
}

impl<I> ReadStage for FakePassthroughStage<I> {
    fn read<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> where C: Context {
        Some(input)
    }
}

impl<I> WriteStage for FakePassthroughStage<I> {
    fn write<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> where C: Context {
        Some(input)
    }
}
