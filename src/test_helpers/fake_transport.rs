use traits::*;

pub struct FakeTransport<'a> {
    buf: &'a mut Vec<u8>,
    pub spawned: bool,
    pub closed: bool,
}

impl<'a> FakeTransport<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> FakeTransport<'a> {
        FakeTransport {
            buf: buf,
            spawned: false,
            closed: false,
        }
    }
}

impl<'t> Transport for FakeTransport<'t> {
    type Buffer = Vec<u8>;

    fn buffer(&mut self) -> &mut Self::Buffer {
        &mut self.buf
    }

    fn spawned(&mut self) {
        self.spawned = true
    }

    fn close(&mut self) {
        self.closed = true
    }

    fn transport_closed(&mut self) {
        self.closed = true
    }

    fn read(&mut self) -> &[u8] {
        &self.buf[..]
    }

    fn consume(&mut self, num: usize) {
        self.buf.clear();
    }

    /// Called when socket changes state to being writable. This method should return any data that
    /// the stage wants to write to the socket.
    fn writable(&mut self) {
    }
}
