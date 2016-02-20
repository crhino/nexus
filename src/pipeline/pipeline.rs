use pipeline::chain::{Chain, Linker};
use pipeline::{Stage};
use std::marker::PhantomData;

pub struct Pipeline<'a, S, Start: 'a> {
    list: Option<Start>,
    socket: S,
    phantom: PhantomData<&'a Start>,
}

impl<'a, S, Start: Chain + Stage<'a, S>> Pipeline<'a, S, Start> {
    pub fn new(socket: S) -> Pipeline<'a, S, Start> {
        Pipeline {
            list: None,
            socket: socket,
            phantom: PhantomData, // Needed for the 'a lifetime parameter on Pipeline to constrain Stage<'a> impl
        }
    }

    pub fn readable(&mut self) {
    }

    pub fn writable(&mut self) {
    }

    pub fn add_stage<St>(self, stage: St) -> Pipeline<'a, S, Linker<St, Start>>
    where St: Stage<'a, S, ReadOutput=Start::ReadInput, WriteInput=Start::WriteOutput> {
            match self {
                Pipeline {
                    list: stages,
                    socket: s,
                    phantom: phantom,
                } => {
                    let stages = prepend_stage(stage, stages);
                    Pipeline {
                        list: Some(stages),
                        socket: s,
                        phantom: PhantomData,
                    }
                }
            }
        }
}

fn recursive_read_write_loop<'a, S, R, W, C>(chain: C, input: R) -> Option<W>
where C: Chain + Stage<'a, S> {
    None
}

fn prepend_stage<'a, S, St, C>(stage: St, chain: Option<C>) -> Linker<St, C>
where St: Stage<'a, S>,
C: Chain + Stage<'a, S, ReadInput=St::ReadOutput, WriteOutput=St::WriteInput>
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

        let mut pipeline = Pipeline::<_, End<Vec<u8>, &[u8]>>::new(Stub).
            add_stage(FakePassthroughStage::<Vec<u8>, &[u8]>::new()).
            add_stage(FakeReadWriteStage::new()).
            add_stage(FakePassthroughStage::<&[u8], &[u8]>::new()).
            add_stage(stage);
        // 2. Initiate a read for pipeline
        pipeline.readable();
        // 3. Last read stage should write back
        // 4. Trigger pipeline write
        pipeline.writable();
        // 5. Assert that write was received
        let written = recv.try_recv().unwrap();

        expect(&written).to(equal(&read));
    }
}
