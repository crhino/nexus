use pipeline::{Context, Stage, ReadStage};
use std::io::{self, Write};
use std::marker::PhantomData;
use future::NexusFuture;

#[derive(Debug)]
pub struct FakeReadWriteStage {
    pub read: Vec<u8>,
    pub write: Vec<u8>,
    pub connected: bool,
    pub closed: bool,
    future: Option<NexusFuture<()>>,
}

impl FakeReadWriteStage {
    pub fn new() -> FakeReadWriteStage {
        FakeReadWriteStage {
            read: Vec::new(),
            write: Vec::new(),
            connected: false,
            closed: false,
            future: None,
        }
    }
}

impl<'a, S> Stage<'a, S> for FakeReadWriteStage {
    type ReadInput = &'a [u8];
    type ReadOutput = Vec<u8>;
    type WriteInput = &'a [u8];
    type WriteOutput = &'a [u8];

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
        self.connected = true;
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.closed = true;
    }

    fn read<C>(&'a mut self, ctx: &mut C, input: Self::ReadInput)
        -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput> {
        let out = input.to_vec();
        let future = ctx.write(input);
        self.future = Some(future);
        Some(out)
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput)
        -> Option<Self::WriteOutput>
            where C: Context<Socket=S> {
        self.write.write_all(input);
        Some(input)
    }
}
