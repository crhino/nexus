use traits::*;
use std::sync::{Arc, Mutex};
use std::io::{self};

pub struct FakeTransport<'a> {
    buf: &'a mut Vec<u8>,
    assertions: Arc<Mutex<TransportAssertions>>,
}

pub struct TransportAssertions {
    pub spawned: bool,
    pub closed: bool,
    pub writable: bool,
}

impl TransportAssertions {
    pub fn new() -> Arc<Mutex<TransportAssertions>> {
        Arc::new(Mutex::new(TransportAssertions {
            spawned: false,
            closed: false,
            writable: false,
        }))
    }
}

impl<'a> FakeTransport<'a> {
    pub fn new(buf: &'a mut Vec<u8>, assertions: Arc<Mutex<TransportAssertions>>) -> FakeTransport<'a> {
        FakeTransport {
            buf: buf,
            assertions: assertions,
        }
    }
}

impl<'t> Transport for FakeTransport<'t> {
    type Buffer = Vec<u8>;

    fn buffer(&mut self) -> &mut Self::Buffer {
        &mut self.buf
    }

    fn spawned(&mut self) {
        let mut a = self.assertions.lock().unwrap();
        a.spawned = true
    }

    fn closed(&mut self, err: Option<&io::Error>) {
        let mut a = self.assertions.lock().unwrap();
        a.closed = true
    }

    fn read(&mut self) -> io::Result<&[u8]> {
        Ok(&self.buf[..])
    }

    fn consume(&mut self, num: usize) {
        self.buf.clear();
    }

    /// Called when socket changes state to being writable. This method should return any data that
    /// the stage wants to write to the socket.
    fn writable(&mut self) {
        let mut a = self.assertions.lock().unwrap();
        a.writable = true
    }
}
