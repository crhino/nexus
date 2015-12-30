use protocol::{Protocol};
use tcp::TcpProtocol;
use mio::tcp::TcpStream;
use mio::{Token, EventSet};
use reactor::{Configurer, ReactorError};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc};

#[derive(Debug, Clone)]
pub struct FakeTcpProtocol {
    connect_count: Arc<AtomicUsize>,
}

impl FakeTcpProtocol {
    pub fn new() -> FakeTcpProtocol {
        FakeTcpProtocol{
            connect_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn connect_count(&self) -> usize {
        self.connect_count.load(Ordering::SeqCst)
    }
}

impl Protocol for FakeTcpProtocol {
    type Socket = TcpStream;

    fn on_readable(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {
    }

    fn on_writable(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {
    }

    fn on_timeout(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {
    }

    fn on_disconnect(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {
    }

    fn on_socket_error(&mut self, _configurer: &mut Configurer<Self::Socket>, _socket: &mut Self::Socket, _token: Token) {
    }

    fn on_event_loop_error(&mut self, error: ReactorError<Self::Socket>) {
        panic!("event loop error: {:?}", error);
    }

    fn tick(&mut self, _configurer: &mut Configurer<Self::Socket>) {
    }
}

impl TcpProtocol for FakeTcpProtocol {
    fn on_connect(&mut self, configurer: &mut Configurer<Self::Socket>, socket: TcpStream) {
        self.connect_count.fetch_add(1, Ordering::SeqCst);
        configurer.add_socket(socket, EventSet::readable());
    }
}
