use future::{Future, Promise, pair};
use traits::*;

pub struct PipelineContext<W> {
    to_write: Option<(W, Promise<()>)>,
}

impl<W> PipelineContext<W> {
    pub fn new() -> PipelineContext<W> {
        PipelineContext {
            to_write: None,
        }
    }

    pub fn into(self) -> Option<(W, Promise<()>)> {
        self.to_write
    }
}

impl<W> Context for PipelineContext<W> {
    type Write = W;

    fn write(&mut self, obj: Self::Write) -> Result<Future<()>, Self::Write> {
        if self.to_write.is_some() {
            return Err(obj)
        }

        let (promise, future) = pair();
        self.to_write = Some((obj, promise));
        Ok(future)
    }

    fn close(&mut self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traits::*;
    use ferrous::dsl::*;

    #[test]
    fn test_write() {
        let mut ctx = PipelineContext::new();
        let write = 9u8;
        let res = ctx.write(write);
        expect(&res).to(be_ok());

        let res = ctx.write(write);
        expect(&res).to(be_err());
        let err = res.unwrap_err();
        expect(&err).to(equal(&9u8));

        let to_write = ctx.into();
        expect(&to_write).to(be_some());
    }
}
