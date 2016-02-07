use pipeline::chain::{Chain, Linker};
use pipeline::{Stage};

pub struct Pipeline<S, Start> {
    socket: S,
    list: Option<Start>,
}

impl<S, Start: Chain> Pipeline<S, Start> {
    pub fn new(socket: S) -> Pipeline<S, Start> {
        Pipeline {
            socket: socket,
            list: None,
        }
    }

    pub fn readable(&mut self) {
    }

    pub fn add_stage<St: Stage>(self, stage: St)
        -> Pipeline<S, Linker<St, Start>> {
            match self {
                Pipeline {
                    socket: s,
                    list: stages,
                } => {
                    let stages = append_stage(stages, stage);
                    Pipeline {
                        socket: s,
                        list: Some(stages),
                    }
                }
            }
        }
}

fn append_stage<S, C1: Chain>(chain: Option<C1>, stage: S) -> Linker<S, C1> {
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
    use test_helpers::{FakeReadStage, FakeReadWriteStage, FakePassthroughStage, FakeWriteStage};
    use rotor::mio::unix::{pipe};
    use std::io::{Write, Read};

    #[test]
    fn test_pipeline_add_read_stage() {
        let (mut r, mut w) = pipe().unwrap();
        let pipeline = Pipeline::<_, ()>::new(r).
            add_stage(ReadOnlyStage::<_, u8>::new(FakeReadStage::new())).
            add_stage(ReadOnlyStage::<_, u8>::new(FakeReadStage::new()));
    }

    #[test]
    fn test_pipeline_add_write_stage() {
        let (mut r, mut w) = pipe().unwrap();
        let pipeline = Pipeline::<_, ()>::new(r).
            add_stage(WriteOnlyStage::<u8, _>::new(FakeWriteStage::new())).
            add_stage(WriteOnlyStage::<u8, _>::new(FakeWriteStage::new()));
    }

    #[test]
    fn test_pipeline_read_write_cycle() {
        // 1. Multiple stage pipeline
        let (mut r, mut w) = pipe().unwrap();
        let mut pipeline = Pipeline::<_, ()>::new(r).
            add_stage(WriteOnlyStage::<u8, _>::new(FakeWriteStage::new())).
            add_stage(FakePassthroughStage::<u8, u8>::new()).
            add_stage(FakeReadWriteStage::new()).
            add_stage(FakePassthroughStage::<u8, u8>::new());
        // 2. Initiate a read for pipeline
        pipeline.readable();
        // 3. Last read stage should write back
        // 4. Trigger pipeline write
        // 5. Assert that write was received
        assert!(false);
    }
}
