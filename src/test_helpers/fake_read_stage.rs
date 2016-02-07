use pipeline::{Context, Stage, ReadStage};
use std::io::{self, Write};
use std::marker::PhantomData;
use future::NexusFuture;

pub struct FakeReadStage<'a> {
    pub read: Vec<u8>,
    pub connected: bool,
    pub closed: bool,
    future: Option<NexusFuture<()>>,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> FakeReadStage<'a> {
    pub fn new() -> FakeReadStage<'a> {
        FakeReadStage {
            read: Vec::new(),
            connected: false,
            closed: false,
            future: None,
            phantom: PhantomData,
        }
    }
}

impl<'a> ReadStage for FakeReadStage<'a> {
    type Input = &'a mut [u8];
    type Output = io::Result<()>;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
        self.connected = true;
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.closed = true;
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> where C: Context {
        // let future = ctx.write(input);
        // self.future = Some(future);
        Some(self.read.write_all(input))
    }
}
