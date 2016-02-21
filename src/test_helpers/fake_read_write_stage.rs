use pipeline::{Context, Stage, ReadStage};
use std::io::{self, Write};
use future::{Future, Promise};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct FakeReadWriteStage {
    pub read: Vec<u8>,
    pub write: Vec<u8>,
    pub connected: bool,
    pub closed: bool,
    future: Option<Future<()>>,
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

    pub fn get_future(&mut self) -> io::Result<()> {
        self.future.take().unwrap().get()
    }
}

impl<S> Stage<S> for Arc<Mutex<FakeReadWriteStage>> {
    type ReadInput = <FakeReadWriteStage as Stage<S>>::ReadInput;
    type ReadOutput = <FakeReadWriteStage as Stage<S>>::ReadOutput;
    type WriteInput = <FakeReadWriteStage as Stage<S>>::WriteInput;
    type WriteOutput = <FakeReadWriteStage as Stage<S>>::WriteOutput;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context<Write=Self::WriteOutput> {
        self.lock().unwrap().connected(ctx)
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.lock().unwrap().closed(ctx)
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput)
        -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput> {
        self.lock().unwrap().read(ctx, input)
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput, promise: Promise<()>)
        -> Option<(Self::WriteOutput, Promise<()>)>
            where C: Context<Socket=S> {
        self.lock().unwrap().write(ctx, input, promise)
    }
}

impl<S> Stage<S> for FakeReadWriteStage {
    type ReadInput = Vec<u8>;
    type ReadOutput = Vec<u8>;
    type WriteInput = Vec<u8>;
    type WriteOutput = Vec<u8>;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
        self.connected = true;
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.closed = true;
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput)
        -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput> {
        let future = ctx.write(input.clone()).unwrap();
        self.future = Some(future);
        Some(input)
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput, promise: Promise<()>)
        -> Option<(Self::WriteOutput, Promise<()>)>
            where C: Context<Socket=S> {
        self.write.write_all(&input[..]);
        Some((input, promise))
    }
}
