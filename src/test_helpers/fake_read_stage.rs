use pipeline::{Context, Stage, ReadStage};
use std::io::{self, Write};
use std::marker::PhantomData;
use future::{Future, Promise};

pub struct FakeReadStage {
    pub read: Vec<u8>,
    pub spawned: bool,
    pub closed: bool,
    future: Option<Future<()>>,
}

impl FakeReadStage {
    pub fn new() -> FakeReadStage {
        FakeReadStage {
            read: Vec::new(),
            spawned: false,
            closed: false,
            future: None,
        }
    }
}

impl<S> ReadStage<S> for FakeReadStage {
    type Input = Vec<u8>;
    type Output = io::Result<()>;

    fn spawned<C>(&mut self, ctx: &mut C) where C: Context {
        self.spawned = true;
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.closed = true;
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::Input)
        -> Option<Self::Output>
            where C: Context<Socket=S> {
        // let future = ctx.write(input);
        // self.future = Some(future);
        Some(self.read.write_all(&input[..]))
    }
}
