use pipeline::{Context, Stage, ReadStage};
use std::io::{self, Write};
use std::marker::PhantomData;
use future::NexusFuture;

pub struct FakeReadStage {
    pub read: Vec<u8>,
    pub connected: bool,
    pub closed: bool,
    future: Option<NexusFuture<()>>,
}

impl FakeReadStage {
    pub fn new() -> FakeReadStage {
        FakeReadStage {
            read: Vec::new(),
            connected: false,
            closed: false,
            future: None,
        }
    }
}

impl<S> ReadStage<S> for FakeReadStage {
    type Input = Vec<u8>;
    type Output = io::Result<()>;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
        self.connected = true;
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
