use pipeline::{Context, Stage};
use std::io::{self, Write};
use future::NexusFuture;
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

impl<'a> Stage<'a> for FakeBaseStage {
    type ReadInput = ();
    type ReadOutput = &'a [u8];
    type WriteInput = &'a [u8];
    type WriteOutput = ();

    fn connected<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
    }

    fn read<C>(&'a mut self, ctx: &mut C, input: Self::ReadInput) -> Option<Self::ReadOutput> where C: Context<Write=Self::WriteOutput> {
        self.vec = self.input.recv().unwrap();
        Some(&mut self.vec[..])
    }

    fn write<C>(&mut self, ctx: &mut C, input: Self::WriteInput) -> Option<Self::WriteOutput> where C: Context {
        let vec = input.to_vec();
        self.output.send(vec).unwrap();
        None
    }
}
