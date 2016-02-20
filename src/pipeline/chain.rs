use pipeline::{Context, Stage, ReadStage, WriteStage};
use void::Void;
use std::marker::PhantomData;

pub trait Chain<'a, S>: Stage<'a, S> {
    type Next: Chain<'a, S> + Stage<'a, S, ReadInput=Self::ReadOutput, WriteOutput=Self::WriteInput>;

    fn add_stage(&mut self, next: Self::Next);
    fn next_stage(&self) -> Option<&Self::Next>;
    fn next_stage_mut(&mut self) -> Option<&mut Self::Next>;
}

#[derive(Debug)]
pub struct End<R, W> {
    read: PhantomData<*const R>,
    write: PhantomData<*const W>,
}

impl<'a, S> Chain<'a, S> for () {
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

impl<'a, S, R: 'a, W: 'a> Chain<'a, S> for End<R, W> {
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

impl<'a, S, S1, S2> Chain<'a, S> for Linker<S1, S2>
where S1: Stage<'a, S>,
      S2: Stage<'a, S, ReadInput=S1::ReadOutput, WriteOutput=S1::WriteInput> + Chain<'a, S> {
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

impl<'a, S> Stage<'a, S> for () {
    type ReadInput = Void;
    type ReadOutput = Void;
    type WriteInput = Void;
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

impl<'a, S, R: 'a, W: 'a> Stage<'a, S> for End<R, W> {
    type ReadInput = R;
    type ReadOutput = Void;
    type WriteInput = Void;
    type WriteOutput = W;

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

impl<'a, S, S1: Stage<'a, S>, S2> Stage<'a, S> for Linker<S1, S2> {
    type ReadInput = S1::ReadInput;
    type ReadOutput = S1::ReadOutput;
    type WriteInput = S1::WriteInput;
    type WriteOutput = S1::WriteOutput;

    fn connected<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S, Write=Self::WriteOutput> {
        self.stage.connected(ctx)
    }

    fn closed<C>(&mut self, ctx: &mut C)
        where C: Context<Socket=S> {
        self.stage.closed(ctx)
    }

    fn write<C>(&'a mut self, ctx: &mut C, input: Self::WriteInput)
        -> Option<Self::WriteOutput>
            where C: Context<Socket=S> {
        self.stage.write(ctx, input)
    }

    fn read<C>(&'a mut self, ctx: &mut C, input: Self::ReadInput)
        -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput> {
        self.stage.read(ctx, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrous::dsl::*;
    use pipeline::{Stage, WriteOnlyStage};
    use test_helpers::{FakePassthroughStage, FakeReadWriteStage, FakeWriteStage};
    use std::borrow::Borrow;
    use std::io;

    struct Stub;
    fn impl_stage<'a, T: Stage<'a, Stub>, S: Borrow<T>>(_: S) {
    }

    #[test]
    fn test_next_stage() {
        let stage = FakePassthroughStage::new();
        let mut linker = Linker::new(stage);

        let next: Linker<_, End<Vec<u8>, &[u8]>> = Linker::new(FakeReadWriteStage::new());
        Chain::<Stub>::add_stage(&mut linker, next);
        let next_stage = Chain::<Stub>::next_stage(&linker);
        expect(&next_stage).to(be_some());

        let last = Chain::<Stub>::next_stage(next_stage.unwrap());
        expect(&last).to(be_none());
    }

    #[test]
    fn test_next_stage_mut() {
        let stage = FakePassthroughStage::new();
        let mut linker = Linker::new(stage);

        let next: Linker<_, End<Vec<u8>, &[u8]>> = Linker::new(FakeReadWriteStage::new());
        Chain::<Stub>::add_stage(&mut linker, next);
        let next_stage = Chain::<Stub>::next_stage(&linker);
        expect(&next_stage).to(be_some());

        let stage = next_stage.unwrap();
    }

    #[test]
    fn test_impl_stage() {
        let stage = FakePassthroughStage::new();
        let mut linker = Linker::new(stage);

        let new_stage = WriteOnlyStage::<u8, _>::new(FakeWriteStage::new());
        let next: Linker<_, End<u8, u8>> = Linker::new(new_stage);
        Chain::<Stub>::add_stage(&mut linker, next);
        impl_stage(linker);
    }
}
