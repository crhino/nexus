use future::{Future, Promise, pair};

pub trait Context {
    type Socket;
    type Write;

    fn socket(&mut self) -> &mut Self::Socket;
    /// The write method can only be called once per stage. The object will be returned if
    /// the object was not scheduled to be written.
    fn write(&mut self, obj: Self::Write) -> Result<Future<()>, Self::Write>;
    fn close(&mut self);
}

pub struct PipelineContext<'a, S: 'a, W> {
    socket: &'a mut S,
    to_write: Option<(W, Promise<()>)>,
}

impl<'a, S: 'a, W> PipelineContext<'a, S, W> {
    pub fn new(socket: &'a mut S) -> PipelineContext<'a, S, W> {
        PipelineContext {
            socket: socket,
            to_write: None,
        }
    }

    pub fn into(self) -> Option<(W, Promise<()>)> {
        self.to_write
    }
}

impl<'a, S: 'a, W> Context for PipelineContext<'a, S, W> {
    type Socket = S;
    type Write = W;

    fn socket(&mut self) -> &mut Self::Socket {
        self.socket
    }

    fn write(&mut self, obj: Self::Write) -> Result<Future<()>, Self::Write> {
        if self.to_write.is_some() {
            return Err(obj)
        }

        let (promise, future) = pair();
        self.to_write = Some((obj, promise));
        Ok(future)
    }

    fn close(&mut self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rotor::mio::unix::{pipe};
    use std::io::{Write, Read};
    use test_helpers::FakeWriteStage;

    #[test]
    fn test_socket() {
        let (mut r, mut w) = pipe().unwrap();

        let buf = [1, 2, 3];
        w.write(&buf).unwrap();
        w.flush().unwrap();

        let mut context = PipelineContext::<_, &[u8]>::new(&mut r);
        let mut socket = context.socket();
        let mut rbuf = [0, 0, 0];
        socket.read(&mut rbuf).unwrap();
        assert_eq!(rbuf, buf);
    }
}
