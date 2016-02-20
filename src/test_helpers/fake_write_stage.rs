use pipeline::{Context, Stage, WriteStage};
use std::io::{self, Write};

#[derive(Debug)]
pub struct FakeWriteStage {
    pub written: Vec<u8>,
    pub connected: bool,
    pub closed: bool,
}

impl FakeWriteStage {
    pub fn new() -> FakeWriteStage {
        FakeWriteStage {
            written: Vec::new(),
            connected: false,
            closed: false,
        }
    }
}

impl<'a, S> WriteStage<'a, S> for FakeWriteStage {
    type Input = u8;
    type Output = io::Result<()>;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
        self.connected = true;
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.closed = true;
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::Input)
        -> Option<Self::Output>
            where C: Context<Socket=S> {
        Some(self.written.write_all(&[input]))
    }
}
