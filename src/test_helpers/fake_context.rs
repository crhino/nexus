use pipeline::{Context};
use future::NexusFuture;

#[derive(Debug)]
pub struct FakeContext;

impl FakeContext {
    pub fn new() -> FakeContext {
        FakeContext
    }
}

impl Context for FakeContext {
    type Socket = ();
    type Write = ();

    fn socket(&mut self) -> &mut Self::Socket {
        unimplemented!()
    }

    fn write(&mut self, obj: Self::Write) -> NexusFuture<()> {
        unimplemented!()
    }

    fn close(&mut self) {
        unimplemented!()
    }
}
