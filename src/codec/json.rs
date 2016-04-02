use serde::{Serialize, Deserialize};
use serde_json::{self, error};

use std::marker::PhantomData;
use std::io::{self, Write};

use traits::*;

pub struct JsonCodec<T> {
    rust_type: PhantomData<* const T>,
}

impl<T> JsonCodec<T> {
    pub fn new() -> JsonCodec<T> {
        JsonCodec {
            rust_type: PhantomData,
        }
    }
}

impl<T: Deserialize + Serialize, B: Write> Codec<B> for JsonCodec<T> {
    type Input = T;
    type Output = error::Result<T>;

    fn encode(&mut self, buffer: &mut B, input: Self::Input) -> io::Result<()> {
        use std::io::ErrorKind::*;
        use std::io::Error;

        serde_json::to_writer(buffer, &input).or_else(|e| {
            Err(Error::new(Other, e))
        })
    }

    fn decode(&mut self, buffer: &[u8]) -> Option<(usize, Self::Output)> {
        Some((buffer.len(), serde_json::from_slice(buffer)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traits::*;
    use ferrous::dsl::*;
    use serde_json::{error};
    use std::io::{Write};

    #[derive(Debug, Copy, Clone, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
    struct Point {
            x: isize,
            y: isize,
    }

    #[test]
    fn test_codec() {
        let mut codec = JsonCodec::new();
        let mut buffer = Vec::new();
        let point = Point { x: 10, y: 100 };

        let res = codec.encode(&mut buffer, point);
        expect(&res).to(be_ok());

        let decode = <JsonCodec<_> as Codec<Vec<u8>>>::decode(&mut codec, &buffer[..]);

        expect(&decode).to(be_some());
        let (_, decoded_output) = decode.unwrap();
        expect(&decoded_output).to(be_ok());

        let decoded_point = decoded_output.unwrap();
        expect(&point).to(equal(&decoded_point));
    }

    #[test]
    fn test_codec_err() {
        let mut codec = JsonCodec::new();
        let json = String::from("{invalid").into_bytes();

        let decode = <JsonCodec<String> as Codec<Vec<u8>>>::decode(&mut codec, &json[..]);
        expect(&decode).to(be_some());

        let (_, decoded_output) = decode.unwrap();
        expect(&decoded_output).to(be_err());
    }
}
