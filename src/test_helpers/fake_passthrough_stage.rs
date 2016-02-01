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

impl<C: Context, I> Stage<C> for FakePassthroughStage<I> {
    type Input = I;
    type Output = I;

    fn connected(&mut self, ctx: &mut C) {
    }

    fn closed(&mut self, ctx: &mut C) {
    }
}

impl<C, I> ReadStage<C> for FakePassthroughStage<I> {
    fn read(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> {
        Some(input)
    }
}

impl<C, I> WriteStage<C> for FakePassthroughStage<I> {
    fn write(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> {
        Some(input)
    }
}
