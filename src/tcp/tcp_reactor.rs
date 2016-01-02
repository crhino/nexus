use reactor::{Reactor, ShutdownHandle, ReactorConfig};
use tcp::protocol::{TcpSocket, ReactorProtocol};
use tcp::{TcpProtocol};
use mio::tcp::{TcpListener};
use mio::{EventSet};
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
        let reactor_proto = ReactorProtocol::new(proto);
        let mut reactor = try!(Reactor::with_configuration(reactor_proto, config));

        let l = TcpSocket::Listener(listener);
        match reactor.add_socket(l, EventSet::readable()) {
            Ok(_) => {},
            Err(e) => {
                return Err(e.into())
            },
        }
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

#[cfg(test)]
mod tests {
    use test_helpers::{FakeTcpProtocol};
    use mio::tcp::{TcpListener, TcpStream};
    use std::thread;
    use std::sync::mpsc::channel;
    use super::*;

    #[test]
    fn test_tcp_run_and_shutdown() {
        let l = TcpListener::bind(&"127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = l.local_addr().unwrap();

        let proto = FakeTcpProtocol::new();
        let mut reactor = TcpReactor::new(proto.clone(), l).unwrap();


        // Another thread that connects to listener
        let stream_thread = thread::spawn(move || {
            let _stream1 = TcpStream::connect(&addr).unwrap();
        });
        stream_thread.join().unwrap();

        let shutdown = reactor.shutdown_handle();
        let (sn, rc) = channel();
        let (done_sn, done_rc) = channel();

        let reactor_thread = thread::spawn(move || {
            rc.recv().unwrap();
            assert!(reactor.run().is_ok());
            done_sn.send(true).unwrap();
        });

        let sht_thread = thread::spawn(move || {
            // Shutdown first so that reactor only spins once.
            shutdown.shutdown();
            sn.send(true).unwrap();
        });

        done_rc.recv().unwrap();

        sht_thread.join().unwrap();
        reactor_thread.join().unwrap();

        assert_eq!(proto.connect_count(), 1);
    }
}
