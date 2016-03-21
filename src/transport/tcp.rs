use traits::*;
use rotor::mio::tcp::TcpStream as MioTcpStream;
use rotor::mio::tcp::Shutdown;
use netbuf::Buf;
use std::io;

pub struct TcpStream {
    stream: MioTcpStream,
    read_buffer: Buf,
}

impl Transport for TcpStream {
    type Buffer = MioTcpStream;

    /// Returns a buffer object that will write data to the underlying socket. This is used by the
    /// Codecs in order to efficiently write data without copying.
    fn buffer(&mut self) -> &mut Self::Buffer {
        &mut self.stream
    }

    fn spawned(&mut self) {
        debug!("spawned tcp stream");
    }

    fn closed(&mut self, err: Option<&io::Error>) {
        debug!("closing tcp stream");
        debug!("transport close: optional error: {:?}", err);

        self.stream.shutdown(Shutdown::Both).map_err(|e| {
            error!("tcp transport: error closing: {}", e);
        });
    }

    fn read(&mut self) -> io::Result<&[u8]> {
        use std::io::ErrorKind::*;

        let stream = &mut self.stream;
        let buf = &mut self.read_buffer;
        loop {
            match buf.read_from(stream) {
                Ok(_) => {},
                Err(e) => {
                    match e.kind() {
                        WouldBlock => {
                            return Ok(&buf[..])
                        },
                        Interrupted => {},
                        _ => {
                            return Err(e)
                        },
                    }
                },
            }
        }
    }

    /// Tells transport that "bytes" number of bytes have been read
    fn consume(&mut self, bytes: usize) {
        self.read_buffer.consume(bytes)
    }

    /// Called when socket changes state to being writable.
    fn writable(&mut self) {
        debug!("writable tcp stream");
    }
}
