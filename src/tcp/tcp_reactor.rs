use reactor::{Reactor, ShutdownHandle, ReactorConfig};
use tcp::protocol::{ReactorProtocol};
use tcp::{TcpProtocol};
use mio::tcp::{TcpListener};
use std::io::{self};

/// A Reactor for TCP connections, handles accepting new connections and passing them to the
/// protocol.
pub struct TcpReactor<P: TcpProtocol> {
    reactor: Reactor<ReactorProtocol<P>>,
}

impl<P: TcpProtocol> TcpReactor<P> {
    /// Create a TcpReactor with the associated listener and default configuration.
    pub fn new(proto: P, listener: TcpListener) -> io::Result<TcpReactor<P>> {
        TcpReactor::with_configuration(proto, listener, ReactorConfig::default())
    }

    /// Create a TcpReactor with the associated listener and specified configuration.
    pub fn with_configuration(proto: P, listener: TcpListener, config: ReactorConfig) -> io::Result<TcpReactor<P>> {
        let reactor_proto = ReactorProtocol::new(proto, listener);
        let reactor = try!(Reactor::with_configuration(reactor_proto, config));
        Ok(TcpReactor{
            reactor: reactor,
        })
    }

    /// Start and run the Reactor.
    pub fn run(&mut self) -> io::Result<()> {
        self.reactor.run()
    }

    /// Handle to shutdown the Reactor.
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        self.reactor.shutdown_handle()
    }

    /// spin_once the Reactor for a single iteration.
    ///
    /// This is mostly used for test purposes.
    pub fn spin_once(&mut self) -> io::Result<()> {
        self.reactor.spin_once()
    }
}
