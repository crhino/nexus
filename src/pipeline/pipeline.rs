use pipeline::chain::{Chain, Linker};
use pipeline::{ReadStage, WriteStage};

pub struct Pipeline<S, RStart, WStart> {
    socket: S,
    read_start: Option<RStart>,
    write_start: Option<WStart>,
}

impl<S, RStart: Chain, WStart: Chain> Pipeline<S, RStart, WStart> {
    pub fn new(socket: S) -> Pipeline<S, RStart, WStart> {
        Pipeline {
            socket: socket,
            read_start: None,
            write_start: None,
        }
    }

    pub fn readable(&mut self) {
    }

    pub fn add_read_stage<C, R: ReadStage<C>>(self, stage: R)
        -> Pipeline<S, Linker<R, RStart>, WStart> {
            match self {
                Pipeline {
                    socket: s,
                    read_start: r,
                    write_start: w
                } => {
                    let read = append_stage(r, stage);
                    Pipeline {
                        socket: s,
                        read_start: Some(read),
                        write_start: w
                    }
                }
            }
        }

    pub fn add_write_stage<C, W: WriteStage<C>>(self, stage: W)
        -> Pipeline<S, RStart, Linker<W, WStart>> {
            match self {
                Pipeline {
                    socket: s,
                    read_start: r,
                    write_start: w
                } => {
                    let write = append_stage(w, stage);
                    Pipeline {
                        socket: s,
                        read_start: r,
                        write_start: Some(write),
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
    use pipeline::{WriteStage, ReadStage};
    use test_helpers::{FakeReadStage, FakePassthroughStage, FakeWriteStage};
    use rotor::mio::unix::{pipe};
    use std::io::{Write, Read};

    #[test]
    fn test_pipeline_add_read_stage() {
        let (mut r, mut w) = pipe().unwrap();
        let pipeline = Pipeline::<_, (), ()>::new(r).
            add_read_stage(FakeReadStage::new()).
            add_read_stage(FakeReadStage::new());
    }

    #[test]
    fn test_pipeline_add_write_stage() {
        let (mut r, mut w) = pipe().unwrap();
        let pipeline = Pipeline::<_, (), ()>::new(r).
            add_write_stage(FakeWriteStage::new()).
            add_write_stage(FakeWriteStage::new());
    }

    #[test]
    fn test_pipeline_read_write_cycle() {
        // 1. Multiple stage pipeline
        let (mut r, mut w) = pipe().unwrap();
        let pipeline = Pipeline::<_, (), ()>::new(r).
            add_write_stage(FakeWriteStage::new()).
            add_write_stage(FakePassthroughStage::new()).
            add_read_stage(FakeReadStage::new()).
            add_read_stage(FakePassthroughStage::new());
        // 2. Initiate a read for pipeline
        pipeline.readable();
        // 3. Last read stage should write back
        // 4. Trigger pipeline write
        // 5. Assert that write was received
        assert!(false);
    }
}
