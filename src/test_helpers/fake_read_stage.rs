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

impl<'a, C: Context> Stage<C> for FakeReadStage<'a> {
    type Input = &'a mut [u8];
    type Output = io::Result<()>;

    fn connected(&mut self, ctx: &mut C) {
        self.connected = true;
    }

    fn closed(&mut self, ctx: &mut C) {
        self.closed = true;
    }
}

impl<'a, C> ReadStage<C> for FakeReadStage<'a>
where C: Context<Write=<FakeReadStage<'a> as Stage<C>>::Input> {
    fn read(&mut self, ctx: &mut C, input: Self::Input) -> Option<Self::Output> {
        let future = ctx.write(input);
        self.future = Some(future);
        Some(self.read.write_all(input))
    }
}
