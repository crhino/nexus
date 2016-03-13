use pipeline::context::PipelineContext;
use future::{Promise};
use void::Void;
use pipeline::{Transport, Codec, Protocol};

pub struct Pipeline<T, C, P> {
    transport: T,
    codec: C,
    protocol: P,
}

impl<'a, T, C, P> Pipeline<T, C, P>
where T: Transport,
      C: Codec<'a, T::Buffer>,
      P: Protocol<'a, Input=C::Output, Output=C::Input>
{
    pub fn new(t: T, c: C, p: P) -> Pipeline<T, C, P> {
        Pipeline {
            transport: t,
            codec: c,
            protocol: p,
        }
    }
}

impl<'a, T, C, P> Pipeline<T, C, P>
where T: Transport,
      C: Codec<'a, T::Buffer>,
      P: Protocol<'a, Input=C::Output, Output=C::Input>
{
    /// Calls spawned method and then writable.
    pub fn spawned(&'a mut self) {
        let mut ctx = PipelineContext::<P::Output>::new();
        self.protocol.spawned(&mut ctx);
        self.transport.spawned();
        self.writable();
    }

    pub fn closed(&mut self) {
        let mut ctx = PipelineContext::<P::Output>::new();
        self.protocol.closed(&mut ctx);
        self.transport.transport_closed();
    }

    pub fn readable(&mut self) {
    }

    /// Signifies that the socket is now writable. This will call transport and
    /// protocol 'writable' method and write any data generated.
    pub fn writable(&'a mut self) {
        let protocol = &mut self.protocol;
        let codec = &mut self.codec;
        let transport = &mut self.transport;

        let mut ctx = PipelineContext::new();
        protocol.writable(&mut ctx);

        match ctx.into() {
            Some((to_write, promise)) => {
                codec.encode(transport.buffer(), to_write, promise);
            },
            None => {}
        }

        transport.writable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrous::dsl::*;
    use std::io::{self, Write, Read};
    use std::sync::{Arc, Mutex};
    use test_helpers::{FakeTransport, FakeCodec, FakeProtocol};

    fn load_protocol_output(proto: &Arc<Mutex<FakeProtocol>>, out: Vec<u8>) {
        let mut p = proto.lock().unwrap();
        p.output.write_all(&out[..]).unwrap();
    }

    #[test]
    fn test_pipeline_writable() {
        let mut vec = vec!(1, 1, 1);
        let transport = FakeTransport::new(&mut vec);
        let codec = FakeCodec::new();
        let protocol = FakeProtocol::new();

        let mut pipeline = Pipeline::new(transport, codec, protocol.clone());
        load_protocol_output(&protocol, vec!(3,3,3));

        // FakeProtocol will send a vec to be written
        pipeline.writable();

        let mut p = protocol.lock().unwrap();
        expect(&(p.future)).to(be_some());
        expect(&(p.future.take().unwrap().get())).to(be_ok());
    }

    #[test]
    fn test_pipeline_spawned() {
        let mut vec = vec!(1, 1, 1);
        let transport = FakeTransport::new(&mut vec);
        let codec = FakeCodec::new();
        let protocol = FakeProtocol::new();

        let mut pipeline = Pipeline::new(transport, codec, protocol.clone());
        load_protocol_output(&protocol, vec!(3,3,3));

        pipeline.spawned();

        let mut p = protocol.lock().unwrap();
        expect(&(p.spawned)).to(equal(&true));
        expect(&(p.future)).to(be_some());
        expect(&(p.future.take().unwrap().get())).to(be_ok());
    }

    #[test]
    fn test_pipeline_closed() {
        let mut vec = vec!(1, 1, 1);
        let transport = FakeTransport::new(&mut vec);
        let codec = FakeCodec::new();
        let protocol = FakeProtocol::new();

        let mut pipeline = Pipeline::new(transport, codec, protocol.clone());
        load_protocol_output(&protocol, vec!(3,3,3));

        pipeline.closed();

        let mut p = protocol.lock().unwrap();
        expect(&(p.closed)).to(equal(&true));
    }

    #[test]
    fn test_pipeline_read_write_cycle() {
        // 1. Multiple stage pipeline
        // let (send, recv, stage) = FakeBaseStage::new();
        // let read = vec!(1,2,3,4,5);
        // send.send(read.clone()).unwrap();

        // let last_stage = Arc::new(Mutex::new(FakeReadWriteStage::new()));

        // let mut pipeline = pipeline(Stub).
        //     add_stage(last_stage.clone()).
        //     add_stage(FakePassthroughStage::<Vec<u8>, Vec<u8>>::new()).
        //     add_stage(stage);
        // 2. Initiate a read for pipeline
        // pipeline.readable();
        assert!(false);
        // 3. Last read stage should write back
        // 5. Assert that write was received
        // let written = recv.try_recv().unwrap();

        // expect(&written).to(equal(&read));
        // expect(&last_stage.lock().unwrap().get_future()).to(be_ok());
    }
}
