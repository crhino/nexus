use pipeline::{Context, Stage, ReadStage};
use std::io::{self, Write};

#[derive(Debug)]
pub struct FakeReadStage {
    pub read: Vec<u8>,
    pub connected: bool,
    pub closed: bool,
}

impl FakeReadStage {
    pub fn new() -> FakeReadStage {
        FakeReadStage {
            read: Vec::new(),
            connected: false,
            closed: false,
        }
    }
}

impl Stage for FakeReadStage {
    type Input = u8;
    type Output = io::Result<()>;

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
        self.connected = true;
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.closed = true;
    }
}

impl ReadStage for FakeReadStage {
    fn read<C>(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> where C: Context {
        Some(self.read.write_all(&[input]))
    }
}
