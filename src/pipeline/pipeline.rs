use pipeline::chain::{Chain, Linker};
use pipeline::{Stage};
use pipeline::context::PipelineContext;
use future::{Promise};
use void::Void;

pub struct Pipeline<S, Start> {
    list: Option<Start>,
    socket: S,
}

impl<S, Start: Chain<S> + Stage<S>> Pipeline<S, Start> {
    pub fn new(socket: S) -> Pipeline<S, Start> {
        Pipeline {
            list: None,
            socket: socket,
        }
    }

    pub fn add_stage<St>(self, stage: St) -> Pipeline<S, Linker<St, Start>>
    where St: Stage<S, ReadOutput=Start::ReadInput, WriteInput=Start::WriteOutput> {
            match self {
                Pipeline {
                    list: stages,
                    socket: s,
                } => {
                    let stages = prepend_stage(stage, stages);
                    Pipeline {
                        list: Some(stages),
                        socket: s,
                    }
                }
            }
        }
}

impl<S, Start: Chain<S, ReadInput=()>> Pipeline<S, Start> {
    pub fn readable(&mut self) {
        let list = &mut self.list;
        let socket = &mut self.socket;

        list.as_mut().and_then(|chain| {
            read_and_write_stages(chain, (), socket)
        });
    }

    // Writable will only call the first stage, since that is the only one that should be writing
    // to the socket.
    pub fn writable(&mut self) {
    }
}

fn read_and_write_stages<S, R, S2, C>(chain: &mut C, input: R, socket: &mut S)
-> Option<(C::WriteOutput, Promise<()>)>
    where C: Chain<S, Next=S2, ReadInput=R>,
          S2: Chain<S, ReadInput=C::ReadOutput, WriteOutput=C::WriteInput> {
    let (read_out, to_write) = {
        let mut ctx = PipelineContext::new(socket);
        let read = chain.read(&mut ctx, input);
        (read, ctx.into())
    };

    // Return early if read stage wants to write, this silently discards read_out var
    // TODO: Is there are better way to handle this case?
    if to_write.is_some() {
        return to_write
    }

    // Recurse until stage doesn't return Some(read_val) or there are no more stages
    read_out.and_then(|read_val| {
        chain.next_stage_mut().and_then(|c| {
            read_and_write_stages(c, read_val, socket)
        })
    }).and_then(|(input, promise)| {
        // Do not let write stage write anything
        let mut ctx = PipelineContext::<_, Void>::new(socket);
        chain.write(&mut ctx, input, promise)
    })
}

fn prepend_stage<S, St, C>(stage: St, chain: Option<C>) -> Linker<St, C>
where St: Stage<S>,
C: Chain<S> + Stage<S, ReadInput=St::ReadOutput, WriteOutput=St::WriteInput>
{
    match chain {
        Some(c) => {
            let mut linker = Linker::new(stage);
            linker.add_stage(c);
            linker
        },
        None => {
            Linker::new(stage)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrous::dsl::*;
    use pipeline::{Stage, ReadOnlyStage, WriteOnlyStage, WriteStage, ReadStage};
    use pipeline::chain::End;
    use test_helpers::{FakeReadStage, FakeReadWriteStage, FakePassthroughStage, FakeWriteStage, FakeBaseStage};
    use std::io::{self, Write, Read};
    use std::sync::{Arc, Mutex};

    struct Stub;

    #[test]
    fn test_pipeline_add_read_stage() {
        let pipeline = Pipeline::<_, End<io::Result<()>, u8>>::new(Stub).
            add_stage(ReadOnlyStage::<_, u8>::new(FakeReadStage::new()));
    }

    #[test]
    fn test_pipeline_add_write_stage() {
        let pipeline = Pipeline::<_, End<u8, u8>>::new(Stub).
            add_stage(WriteOnlyStage::<u8, _>::new(FakeWriteStage::new()));
    }

    #[test]
    fn test_pipeline_read_write_cycle() {
        // 1. Multiple stage pipeline
        let (send, recv, stage) = FakeBaseStage::new();
        let read = vec!(1,2,3,4,5);
        send.send(read.clone()).unwrap();

        let last_stage = Arc::new(Mutex::new(FakeReadWriteStage::new()));

        let mut pipeline = Pipeline::<_, End<Vec<u8>, Vec<u8>>>::new(Stub).
            add_stage(last_stage.clone()).
            add_stage(FakePassthroughStage::<Vec<u8>, Vec<u8>>::new()).
            add_stage(stage);
        // 2. Initiate a read for pipeline
        pipeline.readable();
        // 3. Last read stage should write back
        // 4. Trigger pipeline write
        pipeline.writable();
        // 5. Assert that write was received
        let written = recv.try_recv().unwrap();

        expect(&written).to(equal(&read));
        expect(&last_stage.lock().unwrap().get_future()).to(be_ok());
    }
}
