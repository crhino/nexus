use traits::*;
use std::io::{self, Write};
use future::{Promise};

pub struct FakeCodec {
    pub decoded: Vec<u8>,
    pub encoded: Vec<u8>,
}

impl FakeCodec {
    pub fn new() -> FakeCodec {
        FakeCodec {
            decoded: Vec::new(),
            encoded: Vec::new(),
        }
    }
}

impl Codec<Vec<u8>> for FakeCodec {
    type Input = Vec<u8>;
    type Output = Vec<u8>;

    /// Codec should write encoded data to buffer and finish the promise.
    fn encode(&mut self, buffer: &mut Vec<u8>, input: Self::Input, promise: Promise<()>) -> io::Result<()> {
        self.encoded.write_all(&input[..]).unwrap();
        promise.set(buffer.write_all(&input[..]));
        Ok(())
    }

    // If decode returns None that means the Codec needs more data, otherwise it returns a tuple of
    // the number of bytes used and an Output object.
    fn decode(&mut self, buffer: &[u8]) -> Option<(usize, Self::Output)> {
        self.decoded.write_all(buffer).unwrap();
        Some((buffer.len(), buffer.to_vec()))
    }
}
