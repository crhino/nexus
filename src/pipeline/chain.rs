use pipeline::{Context, Stage, ReadStage, WriteStage};
use void::Void;
use std::marker::PhantomData;

pub trait Chain {
    type Next: Chain;

    fn add_stage(&mut self, next: Self::Next);
    fn next_stage(&self) -> Option<&Self::Next>;
    fn next_stage_mut(&mut self) -> Option<&mut Self::Next>;
}

pub struct End<R, W> {
    read: PhantomData<*const R>,
    write: PhantomData<*const W>,
}

impl Chain for () {
    type Next = ();

    fn add_stage(&mut self, _next: Self::Next) {
    }

    fn next_stage(&self) -> Option<&Self::Next> {
        None
    }

    fn next_stage_mut(&mut self) -> Option<&mut Self::Next> {
        None
    }
}

impl<R, W> Chain for End<R, W> {
    type Next = ();

    fn add_stage(&mut self, _next: Self::Next) {
    }

    fn next_stage(&self) -> Option<&Self::Next> {
        None
    }

    fn next_stage_mut(&mut self) -> Option<&mut Self::Next> {
        None
    }
}

impl<S1, S2: Chain> Chain for Linker<S1, S2> {
    type Next = S2;

    fn add_stage(&mut self, next: Self::Next) {
        self.next = Some(next);
    }

    fn next_stage(&self) -> Option<&Self::Next> {
        self.next.as_ref()
    }

    fn next_stage_mut(&mut self) -> Option<&mut Self::Next> {
        self.next.as_mut()
    }
}

#[derive(Debug)]
pub struct Linker<S1, S2> {
    stage: S1,
    next: Option<S2>,
}

impl<S1, S2> Linker<S1, S2> {
    pub fn new(stage: S1) -> Linker<S1, S2> {
        Linker {
            stage: stage,
            next: None,
        }
    }
}

impl<R, W> Stage for End<R, W> {
    type ReadInput = R;
    type ReadOutput = Void;
    type WriteInput = W;
    type WriteOutput = Void;

    fn connected<C>(&mut self, _ctx: &mut C) where C: Context {
        unreachable!()
    }

    fn closed<C>(&mut self, _ctx: &mut C) where C: Context {
        unreachable!()
    }

    fn write<C>(&mut self, _ctx: &mut C, input: Self::WriteInput) -> Option<Self::WriteOutput> where C: Context {
        unreachable!()
    }

    fn read<C>(&mut self, _ctx: &mut C, input: Self::ReadInput) -> Option<Self::ReadOutput> where C: Context<Write=Self::WriteOutput> {
        unreachable!()
    }
}

impl<S1: Stage, S2> Stage for Linker<S1, S2> {
    type ReadInput = S1::ReadInput;
    type ReadOutput = S1::ReadOutput;
    type WriteInput = S1::WriteInput;
    type WriteOutput = S1::WriteOutput;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
        self.stage.connected(ctx)
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.stage.closed(ctx)
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput) -> Option<Self::WriteOutput> where C: Context {
        self.stage.write(ctx, input)
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput) -> Option<Self::ReadOutput> where C: Context<Write=Self::WriteOutput> {
        self.stage.read(ctx, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrous::dsl::*;
    use pipeline::{Stage, WriteOnlyStage};
    use test_helpers::{FakeWriteStage};
    use std::borrow::Borrow;

    fn impl_stage<T: Stage, S: Borrow<T>>(_: S) {
    }

    #[test]
    fn test_next_stage() {
        let stage = FakeWriteStage::new();
        let mut linker = Linker::new(stage);

        let next: Linker<_, ()> = Linker::new(FakeWriteStage::new());
        linker.add_stage(next);
        let next_stage = linker.next_stage();
        expect(&next_stage).to(be_some());

        let last = next_stage.unwrap().next_stage();
        expect(&last).to(be_none());
    }

    #[test]
    fn test_next_stage_mut() {
        let stage = FakeWriteStage::new();
        let mut linker = Linker::new(stage);

        let next: Linker<_, ()> = Linker::new(FakeWriteStage::new());
        linker.add_stage(next);
        let next_stage = linker.next_stage_mut();
        expect(&next_stage).to(be_some());

        let stage = next_stage.unwrap();
    }

    #[test]
    fn test_impl_stage() {
        let stage = WriteOnlyStage::<u8, _>::new(FakeWriteStage::new());
        let mut linker = Linker::new(stage);

        let new_stage = WriteOnlyStage::<u8, _>::new(FakeWriteStage::new());
        let next: Linker<_, ()> = Linker::new(new_stage);
        linker.add_stage(next);
        impl_stage(linker);
    }
}
