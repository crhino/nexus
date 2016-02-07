use pipeline::chain::{Chain, Linker};
use pipeline::{Stage};

pub struct Pipeline<Start> {
    list: Option<Start>,
}

impl<Start: Chain + Stage> Pipeline<Start> {
    pub fn new() -> Pipeline<Start> {
        Pipeline {
            list: None,
        }
    }

    pub fn readable(&mut self) {
    }

    pub fn writable(&mut self) {
    }

    pub fn add_stage<S>(self, stage: S) -> Pipeline<Linker<S, Start>>
    where S: Stage<ReadOutput=Start::ReadInput, WriteOutput=Start::WriteInput> {
            match self {
                Pipeline {
                    list: stages,
                } => {
                    let stages = prepend_stage(stage, stages);
                    Pipeline {
                        list: Some(stages),
                    }
                }
            }
        }
}

fn prepend_stage<S, C>(stage: S, chain: Option<C>) -> Linker<S, C>
where S: Stage,
C: Chain + Stage<ReadInput=S::ReadOutput, WriteInput=S::WriteOutput>
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
    use test_helpers::{FakeReadStage, FakeReadWriteStage, FakePassthroughStage, FakeWriteStage};
    use std::io::{self, Write, Read};

    #[test]
    fn test_pipeline_add_read_stage() {
        let pipeline = Pipeline::<End<io::Result<()>, u8>>::new().
            add_stage(ReadOnlyStage::<_, u8>::new(FakeReadStage::new()));
    }

    #[test]
    fn test_pipeline_add_write_stage() {
        let pipeline = Pipeline::<End<u8, io::Result<()>>>::new().
            add_stage(WriteOnlyStage::<u8, _>::new(FakeWriteStage::new()));
    }

    #[test]
    fn test_pipeline_read_write_cycle() {
        // 1. Multiple stage pipeline
        // let (send, recv, stage) = FakeBaseStage::new();

        let mut pipeline = Pipeline::<End<Vec<u8>, &mut [u8]>>::new().
            add_stage(FakePassthroughStage::<Vec<u8>, &mut [u8]>::new()).
            add_stage(FakeReadWriteStage::new()).
            add_stage(FakePassthroughStage::<&mut [u8], &mut [u8]>::new());
            // add_stage(stage);
        // 2. Initiate a read for pipeline
        pipeline.readable();
        // 3. Last read stage should write back
        // 4. Trigger pipeline write
        pipeline.writable();
        // 5. Assert that write was received
        assert!(false);
    }
}
