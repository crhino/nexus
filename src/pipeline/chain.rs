use pipeline::{Context, Stage, ReadStage, WriteStage};

pub trait Chain {
    type Next: Chain;

    fn add_stage(&mut self, next: Self::Next);
    fn next_stage(&self) -> Option<&Self::Next>;
    fn next_stage_mut(&mut self) -> Option<&mut Self::Next>;
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

impl<S1: Stage, S2> Stage for Linker<S1, S2> {
    type Input = S1::Input;
    type Output = S1::Output;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
        self.stage.connected(ctx)
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.stage.closed(ctx)
    }
}

impl<S1: WriteStage, S2> WriteStage for Linker<S1, S2> {
    fn write<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> where C: Context {
        self.stage.write(ctx, input)
    }
}

impl<S1: ReadStage, S2> ReadStage for Linker<S1, S2> {
    fn read<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> where C: Context {
        self.stage.read(ctx, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrous::dsl::*;
    use pipeline::{WriteStage, ReadStage};
    use test_helpers::{FakeReadStage, FakeWriteStage};
    use std::borrow::Borrow;

    fn impl_write_stage<T: WriteStage, S: Borrow<T>>(_: S) {
    }

    fn impl_read_stage<T: ReadStage, S: Borrow<T>>(_: S) {
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
        impl_write_stage::<Linker<FakeWriteStage, _>, _>(stage);
    }

    #[test]
    fn test_impl_write_stage() {
        let stage = FakeWriteStage::new();
        let mut linker = Linker::new(stage);

        let next: Linker<_, ()> = Linker::new(FakeWriteStage::new());
        linker.add_stage(next);
        impl_write_stage(linker);
    }

    #[test]
    fn test_impl_read_stage() {
        let stage = FakeReadStage::new();
        let mut linker = Linker::new(stage);

        let next: Linker<_, ()> = Linker::new(FakeReadStage::new());
        linker.add_stage(next);
        impl_read_stage(linker);
    }
}
