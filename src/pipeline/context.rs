use future::NexusFuture;
use std::marker::PhantomData;

pub trait Context {
    type Socket;
    type Write;

    fn socket(&mut self) -> &mut Self::Socket;
    fn write(&mut self, obj: Self::Write) -> NexusFuture<()>;
    fn close(&mut self);
}

pub struct PipelineContext<'a, S: 'a, W> {
    socket: &'a mut S,
    write: PhantomData<*const W>,
}

impl<'a, S: 'a, W> PipelineContext<'a, S, W> {
    pub fn new(socket: &'a mut S) -> PipelineContext<'a, S, W> {
        PipelineContext {
            socket: socket,
            write: PhantomData,
        }
    }
}

impl<'a, S: 'a, W> Context for PipelineContext<'a, S, W> {
    type Socket = S;
    type Write = W;

    fn socket(&mut self) -> &mut Self::Socket {
        self.socket
    }

    fn write(&mut self, obj: Self::Write) -> NexusFuture<()> {
        unimplemented!()
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
