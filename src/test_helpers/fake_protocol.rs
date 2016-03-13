use std::sync::{Arc, Mutex};
use future::{Future};
use pipeline::{Context, Protocol};
use std::io::{Write};

pub struct FakeProtocol {
    pub input: Vec<u8>,
    pub output: Vec<u8>,
    pub future: Option<Future<()>>,
    pub spawned: bool,
    pub closed: bool,
}

impl FakeProtocol {
    pub fn new() -> Arc<Mutex<FakeProtocol>> {
        Arc::new(Mutex::new(FakeProtocol {
            input: Vec::new(),
            output: Vec::new(),
            future: None,
            spawned: false,
            closed: false,
        }))
    }
}

impl Protocol for Arc<Mutex<FakeProtocol>> {
    type Input = Vec<u8>;
    type Output = Vec<u8>;

    fn spawned<C>(&mut self, ctx: &mut C) where C: Context {
        self.lock().unwrap().spawned = true;
    }

    fn closed<C>(&mut self, ctx: &mut C) where C: Context {
        self.lock().unwrap().closed = true;
    }

    fn received_data<C>(&mut self, ctx: &mut C, data: Self::Input) where C: Context<Write=Self::Output> {
        let mut p = self.lock().unwrap();
        p.input.write_all(&data[..]).unwrap();
        let f = ctx.write(p.output.clone()).unwrap();
        p.future = Some(f);
    }

    /// Called when socket changes state to being writable.
    fn writable<C>(&mut self, ctx: &mut C) where C: Context<Write=Self::Output> {
        let mut p = self.lock().unwrap();
        let f = ctx.write(p.output.clone()).unwrap();
        p.future = Some(f);
    }
}
