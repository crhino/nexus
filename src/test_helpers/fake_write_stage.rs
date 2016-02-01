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

impl<C> Stage<C> for FakeWriteStage {
    type Input = u8;
    type Output = io::Result<()>;

    fn connected(&mut self, ctx: &mut C) {
        self.connected = true;
    }

    fn closed(&mut self, ctx: &mut C) {
        self.closed = true;
    }
}

impl<C> WriteStage<C> for FakeWriteStage {
    fn write(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> {
        Some(self.written.write_all(&[input]))
    }
}
