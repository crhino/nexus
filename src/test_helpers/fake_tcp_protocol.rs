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

    fn on_readable<C>(&mut self, _configurer: &mut C, _socket: &mut Self::Socket, _token: Token) where C: Configurer<Self::Socket> {}

    fn on_writable<C>(&mut self, _configurer: &mut C, _socket: &mut Self::Socket, _token: Token) where C: Configurer<Self::Socket> {}

    fn on_timeout<C>(&mut self, _configurer: &mut C, _socket: &mut Self::Socket, _token: Token) where C: Configurer<Self::Socket> {}

    fn on_disconnect<C>(&mut self, _configurer: &mut C, _socket: &mut Self::Socket, _token: Token) where C: Configurer<Self::Socket> {}

    fn on_socket_error<C>(&mut self, _configurer: &mut C, _socket: &mut Self::Socket, _token: Token) where C: Configurer<Self::Socket> {}

    fn on_event_loop_error(&mut self, error: ReactorError<Self::Socket>) {
        panic!("fake tcp event loop error: {:?}", error)
    }

    fn tick<C>(&mut self, _configurer: &mut C) where C: Configurer<Self::Socket> {}
}

impl TcpProtocol for FakeTcpProtocol {
    fn on_connect<C>(&mut self,
                     configurer: &mut C,
                     socket: TcpStream) where C: Configurer<Self::Socket> {
        self.connect_count.fetch_add(1, Ordering::SeqCst);
        configurer.add_socket(socket, EventSet::readable());
    }
}
