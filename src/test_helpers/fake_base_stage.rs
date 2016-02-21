use pipeline::{Context, Stage};
use std::io::{self, Write};
use future::{Future, Promise};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::marker::PhantomData;
use void::Void;

pub struct FakeBaseStage {
    input: Receiver<Vec<u8>>,
    output: Sender<Vec<u8>>,
    vec: Vec<u8>,
}

impl FakeBaseStage {
    pub fn new() -> (Sender<Vec<u8>>, Receiver<Vec<u8>>, FakeBaseStage) {
        let (in_sn, in_rc) = channel();
        let (out_sn, out_rc) = channel();
        let stage = FakeBaseStage {
            input: in_rc,
            output: out_sn,
            vec: Vec::new(),
        };
        (in_sn, out_rc, stage)
    }
}

impl<S> Stage<S> for FakeBaseStage {
    type ReadInput = ();
    type ReadOutput = Vec<u8>;
    type WriteInput = Vec<u8>;
    type WriteOutput = ();

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn read<C>(&mut self, ctx: &mut C, input: Self::ReadInput)
        -> Option<Self::ReadOutput>
            where C: Context<Socket=S, Write=Self::WriteOutput> {
        self.vec = self.input.recv().unwrap();
        Some(self.vec.clone())
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput, promise: Promise<()>)
        -> Option<(Self::WriteOutput, Promise<()>)>
            where C: Context<Socket=S> {
        self.output.send(input).unwrap();
        None
    }
}
