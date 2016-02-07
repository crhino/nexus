use pipeline::WriteStage;
use future::NexusFuture;

pub trait Context {
    type Socket;
    type Write;

    fn socket(&mut self) -> &mut Self::Socket;
    fn write(&mut self, obj: Self::Write) -> NexusFuture<()>;
    fn close(&mut self);
}

// pub struct PipelineContext<'a, S: 'a, W: 'a> {
//     socket: &'a mut S,
//     write_stage: &'a mut W,
// }

// impl<'a, S: 'a, W: 'a> PipelineContext<'a, S, W> {
//     pub fn new(socket: &'a mut S, write_stage: &'a mut W) -> PipelineContext<'a, S, W> {
//         PipelineContext {
//             socket: socket,
//             write_stage: write_stage,
//         }
//     }
// }

// impl<'a, S: 'a, W> Context for PipelineContext<'a, S, W> {
//     type Socket = S;
//     type Write = W::Input;

//     fn socket(&mut self) -> &mut Self::Socket {
//         self.socket
//     }

//     fn write(&mut self, obj: Self::Write) -> NexusFuture<()> {
//         unimplemented!()
//     }

//     fn close(&mut self) {
//         unimplemented!()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use rotor::mio::unix::{pipe};
//     use std::io::{Write, Read};
//     use test_helpers::FakeWriteStage;

//     #[test]
//     fn test_socket() {
//         let (mut r, mut w) = pipe().unwrap();

//         let buf = [1, 2, 3];
//         w.write(&buf).unwrap();
//         w.flush().unwrap();

//         let mut stage = FakeWriteStage::new();

//         let mut context = PipelineContext::new(&mut r, &mut stage);
//         let mut socket = context.socket();
//         let mut rbuf = [0, 0, 0];
//         socket.read(&mut rbuf).unwrap();
//         assert_eq!(rbuf, buf);
//     }
// }
