use std::io::{self, Write};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

use traits::*;

const DEFAULT_CAPACITY: usize = 1024;

pub struct FixedLengthCodec<C> {
    codec: C,
    buffer: Vec<u8>,
}

impl<C> FixedLengthCodec<C> {
    pub fn new(codec: C) -> FixedLengthCodec<C> {
        FixedLengthCodec {
            codec: codec,
            buffer: Vec::with_capacity(DEFAULT_CAPACITY),
        }
    }
}

impl<C: Codec<Vec<u8>>, B: Write> Codec<B> for FixedLengthCodec<C> {
    type Input = C::Input;
    type Output = C::Output;

    fn encode(&mut self, buffer: &mut B, input: Self::Input) -> io::Result<()> {
        let codec = &mut self.codec;
        let inner_buf = &mut self.buffer;
        try!(codec.encode(inner_buf, input));
        let len = inner_buf.len();
        debug_assert!(len <= ::std::u32::MAX as usize);

        try!(buffer.write_u32::<BigEndian>(len as u32));
        buffer.write_all(&inner_buf[..])
    }

    fn decode(&mut self, mut buffer: &[u8]) -> Option<(usize, Self::Output)> {
        // Not enough bytes to read u32
        if buffer.len() < 4 {
            return None;
        }

        let len = match buffer.read_u32::<BigEndian>() {
            Err(_) => {
                warn!("failed to read u32 value");
                return None
            },
            Ok(n) => n,
        };

        if buffer.len() < (len as usize) {
            return None
        }

        self.codec.decode(&buffer[..len as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traits::*;
    use ferrous::dsl::*;
    use std::io::{Write};
    use test_helpers::FakeCodec;

    #[test]
    fn test_codec() {
        let fake = FakeCodec::new();
        let mut codec = FixedLengthCodec::new(fake);
        let mut buffer = Vec::new();
        let input = vec!(1,2,3,4,5,6,7,8,9);

        let res = codec.encode(&mut buffer, input.clone());
        expect(&res).to(be_ok());
        println!("{:?}", buffer);
        expect(&buffer[3]).to(equal(&9));

        let decode = <FixedLengthCodec<_> as Codec<Vec<u8>>>::decode(&mut codec, &buffer[..]);

        expect(&decode).to(be_some());
        let (_, decoded_output) = decode.unwrap();
        expect(&decoded_output).to(equal(&input));
    }

    #[test]
    fn test_codec_not_enough_bytes() {
        let fake = FakeCodec::new();
        let mut codec = FixedLengthCodec::new(fake);
        let mut buffer = Vec::new();
        let input = vec!(1,2,3,4,5,6,7,8,9);

        let res = codec.encode(&mut buffer, input);
        expect(&res).to(be_ok());

        let decode = <FixedLengthCodec<_> as Codec<Vec<u8>>>::decode(&mut codec, &buffer[..1]);
        expect(&decode).to(be_none());
    }

    #[test]
    fn test_codec_less_than_length() {
        let fake = FakeCodec::new();
        let mut codec = FixedLengthCodec::new(fake);
        let mut buffer = Vec::new();
        let input = vec!(1,2,3,4,5,6,7,8,9);

        let res = codec.encode(&mut buffer, input);
        expect(&res).to(be_ok());

        let decode = <FixedLengthCodec<_> as Codec<Vec<u8>>>::decode(&mut codec, &buffer[..8]);
        expect(&decode).to(be_none());
    }
}
