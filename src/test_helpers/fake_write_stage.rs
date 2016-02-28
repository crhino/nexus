use pipeline::{Context, Stage, WriteStage};
use future::{Promise};
use std::io::{self, Write};

#[derive(Debug)]
pub struct FakeWriteStage {
    pub written: Vec<u8>,
    pub spawned: bool,
    pub closed: bool,
}

impl FakeWriteStage {
    pub fn new() -> FakeWriteStage {
        FakeWriteStage {
            written: Vec::new(),
            spawned: false,
            closed: false,
        }
    }
}

impl<S> WriteStage<S> for FakeWriteStage {
    type Input = u8;
    type Output = io::Result<()>;

    fn spawned<C>(&mut self, ctx: &mut C) where C: Context {
        self.spawned = true;
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.closed = true;
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::Input, promise: Promise<()>)
        -> Option<(Self::Output, Promise<()>)>
            where C: Context<Socket=S> {
        Some((self.written.write_all(&[input]), promise))
    }

    fn writable<C>(&mut self, ctx: &mut C)
        -> Option<(Self::Output, Promise<()>)>
            where C: Context<Socket=S> {
                None
            }
}
