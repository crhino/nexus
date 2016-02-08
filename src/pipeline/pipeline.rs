use pipeline::chain::{Chain, Linker};
use pipeline::{Stage};
use std::marker::PhantomData;

pub struct Pipeline<'a, Start: 'a> {
    list: Option<Start>,
    phantom: PhantomData<&'a Start>,
}

impl<'a, Start: Chain + Stage<'a>> Pipeline<'a, Start> {
    pub fn new() -> Pipeline<'a, Start> {
        Pipeline {
            list: None,
            phantom: PhantomData, // Needed for the 'a lifetime parameter on Pipeline to constrain Stage<'a> impl
        }
    }

    pub fn readable(&mut self) {
    }

    pub fn writable(&mut self) {
    }

    pub fn add_stage<S>(self, stage: S) -> Pipeline<'a, Linker<S, Start>>
    where S: Stage<'a, ReadOutput=Start::ReadInput, WriteInput=Start::WriteOutput> {
            match self {
                Pipeline {
                    list: stages,
                    phantom: phantom,
                } => {
                    let stages = prepend_stage(stage, stages);
                    Pipeline {
                        list: Some(stages),
                        phantom: PhantomData,
                    }
                }
            }
        }
}

fn recursive_read_write_loop<'a, R, W, C>(chain: C, input: R) -> Option<W>
where C: Chain + Stage<'a> {
}

fn prepend_stage<'a, S, C>(stage: S, chain: Option<C>) -> Linker<S, C>
where S: Stage<'a>,
C: Chain + Stage<'a, ReadInput=S::ReadOutput, WriteOutput=S::WriteInput>
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

    #[test]
    fn test_pipeline_add_read_stage() {
        let pipeline = Pipeline::<End<io::Result<()>, u8>>::new().
            add_stage(ReadOnlyStage::<_, u8>::new(FakeReadStage::new()));
    }

    #[test]
    fn test_pipeline_add_write_stage() {
        let pipeline = Pipeline::<End<u8, u8>>::new().
            add_stage(WriteOnlyStage::<u8, _>::new(FakeWriteStage::new()));
    }

    #[test]
    fn test_pipeline_read_write_cycle() {
        // 1. Multiple stage pipeline
        let (send, recv, stage) = FakeBaseStage::new();
        let read = vec!(1,2,3,4,5);
        send.send(read.clone()).unwrap();

        let mut pipeline = Pipeline::<End<Vec<u8>, &[u8]>>::new().
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
