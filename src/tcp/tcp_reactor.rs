use reactor::{Reactor, ReactorConfig};
use tcp::protocol::{ReactorProtocol};
use protocol::{Protocol};
use tcp::{TcpProtocol};
use mio::tcp::{TcpListener};
use mio::{Evented};
use std::io::{self};

pub struct TcpReactor<P: TcpProtocol> {
    reactor: Reactor<ReactorProtocol<P>>,
}

impl<P: TcpProtocol> TcpReactor<P> {
    pub fn new(proto: P, listener: TcpListener) -> io::Result<TcpReactor<P>> {
        TcpReactor::with_configuration(proto, listener, ReactorConfig::default())
    }

    pub fn with_configuration(proto: P, listener: TcpListener, config: ReactorConfig) -> io::Result<TcpReactor<P>> {
        let reactor_proto = ReactorProtocol::new(proto, listener);
        let reactor = try!(Reactor::with_configuration(reactor_proto, config));
        Ok(TcpReactor{
            reactor: reactor,
        })
    }

    // TODO: Add start and shutdown methods
}
