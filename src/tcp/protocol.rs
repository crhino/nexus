use protocol::Protocol;
use reactor::{Token, Configurer, ReactorError};
use mio::tcp::{TcpStream, TcpListener};

/// Trait used with a TCP reactor.
pub trait TcpProtocol: Protocol {
    /// Event called when a new tcp connections is received.
    ///
    /// The socket should be transformed into `Self::Socket` type and added to the reactor with the
    /// configurer.
    fn on_connect(&mut self, configurer: &mut Configurer<Self::Socket>, socket: TcpStream);
}

pub struct ReactorProtocol<P> {
    protocol: P,
    listener: TcpListener,
}

impl<P> ReactorProtocol<P> {
    pub fn new(proto: P, listener: TcpListener) -> ReactorProtocol<P> {
        ReactorProtocol{
            protocol: proto,
            listener: listener,
        }
    }
}

impl<P: TcpProtocol> TcpProtocol for ReactorProtocol<P> {
    fn on_connect(&mut self, configurer: &mut Configurer<Self::Socket>, socket: TcpStream) {
        self.protocol.on_connect(configurer, socket);
    }
}

impl<P: TcpProtocol> Protocol for ReactorProtocol<P> {
    type Socket = P::Socket;

    fn on_readable(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token) {
        self.protocol.on_readable(configurer, socket, token)
    }

    fn on_writable(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token) {
        self.protocol.on_writable(configurer, socket, token)
    }

    fn on_timeout(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token) {
        self.protocol.on_timeout(configurer, socket, token)
    }

    fn on_disconnect(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token) {
        self.protocol.on_disconnect(configurer, socket, token)
    }

    fn on_socket_error(&mut self, configurer: &mut Configurer<Self::Socket>, socket: &mut Self::Socket, token: Token) {
        self.protocol.on_socket_error(configurer, socket, token)
    }

    fn on_event_loop_error(&mut self, error: ReactorError<Self::Socket>) {
        self.protocol.on_event_loop_error(error)
    }

    fn tick(&mut self, configurer: &mut Configurer<Self::Socket>) {
        loop {
            let l = &mut self.listener;
            let proto = &mut self.protocol;
            match l.accept() {
                Ok(Some((skt, _addr))) => {
                    proto.on_connect(configurer, skt);
                },
                Ok(None) => break,
                Err(e) => {
                    error!("error accepting connections: {:?}", e);
                    return
                    // TODO: Initate shutdown
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mio::tcp::{TcpListener, TcpStream};
    use test_helpers::{FakeTcpProtocol};
    use protocol::Protocol;
    use std::thread;
    use super::ReactorProtocol;
    use reactor::configurer::{ProtocolConfigurer};

    #[test]
    fn test_adding_new_connections() {
        let l = TcpListener::bind(&"127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = l.local_addr().unwrap();
        let proto = FakeTcpProtocol::new();
        let mut reactor_proto = ReactorProtocol::new(proto.clone(), l);
        let mut configurer = ProtocolConfigurer::new();

        let handle = thread::spawn(move || {
            let _stream1 = TcpStream::connect(&addr).unwrap();
            let _stream2 = TcpStream::connect(&addr).unwrap();
        });

        handle.join().unwrap();
        reactor_proto.tick(&mut configurer);

        assert_eq!(proto.connect_count(), 2);
    }
}
