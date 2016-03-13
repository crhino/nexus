use pipeline::context::PipelineContext;
use future::{Promise};
use void::Void;
use pipeline::{Transport, Codec, Protocol};

pub struct Pipeline<T, C, P> {
    transport: T,
    codec: C,
    protocol: P,
}

impl<T, C, P> Pipeline<T, C, P>
where T: Transport,
      C: Codec<T::Buffer>,
      P: Protocol<Input=C::Output, Output=C::Input>
{
    pub fn new(t: T, c: C, p: P) -> Pipeline<T, C, P> {
        Pipeline {
            transport: t,
            codec: c,
            protocol: p,
        }
    }
}

impl<T, C, P> Pipeline<T, C, P>
where T: Transport,
      C: Codec<T::Buffer>,
      P: Protocol<Input=C::Output, Output=C::Input>
{
    /// Calls spawned method and then writable.
    pub fn spawned(&mut self) {
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

    fn read_data(&mut self) -> Option<(C::Input, Promise<()>)> {
        let (num, output) = {
            let read = self.transport.read();
            let decoded = self.codec.decode(read);
            match decoded {
                Some(d) => d,
                None => return None,
            }
        };
        self.transport.consume(num);

        let mut ctx = PipelineContext::new();
        self.protocol.received_data(&mut ctx, output);

        ctx.into()
    }

    pub fn readable(&mut self) {
        self.read_data().map(|(to_write, promise)| {
            self.codec.encode(self.transport.buffer(), to_write, promise);
        });
    }

    /// Signifies that the socket is now writable. This will call transport and
    /// protocol 'writable' method and write any data generated.
    pub fn writable(&mut self) {
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
        let mut vec = vec!(1, 1, 1);
        let protocol = FakeProtocol::new();
        {
            let transport = FakeTransport::new(&mut vec);
            let codec = FakeCodec::new();

            let mut pipeline = Pipeline::new(transport, codec, protocol.clone());
            load_protocol_output(&protocol, vec!(3,3,3));

            // 2. Initiate a read for pipeline
            pipeline.readable();
            // 3. protocol should write back
        }

        // 5. Assert that write was received
        // Expect that initial vector is consumed by transport
        let expected = vec!(3,3,3);
        expect(&expected).to(equal(&vec));

        let mut p = protocol.lock().unwrap();
        expect(&(p.future)).to(be_some());
        expect(&(p.future.take().unwrap().get())).to(be_ok());
    }
}
