pub mod configurer;
mod handler; pub use reactor::configurer::{Configurer};
pub use mio::Token;
pub use reactor::handler::ReactorHandler;

use mio::{EventLoop, EventLoopConfig};
use std::io::{self};
use std::error::Error;
use std::fmt;

use protocol::Protocol;

const SLAB_GROW_SIZE: usize = 1024;

/// A Reactor runs the event loop and manages sockets
pub struct Reactor<P: Protocol>(EventLoop<ReactorHandler<P>>, ReactorHandler<P>);

/// Configuration for the Reactor
pub struct ReactorConfig {
    timer_capacity: usize,
    timer_tick_interval_ms: Option<u64>,
}

#[derive(Debug)]
/// Error returned by the Reactor.
pub enum ReactorError<S> {
    /// An I/O error was returned from the OS.
    IoError(io::Error, S),
    /// Could not find associated socket for the token.
    NoSocketFound(Token),
    /// An error occurred while adding a timeout.
    TimerError,
}

impl<S> fmt::Display for ReactorError<S> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReactorError::IoError(ref e, _) => {
                write!(fmt, "io error: {}", e)
            },
            ReactorError::NoSocketFound(token) => {
                write!(fmt, "could not find socket with associated token {:?}", token)
            },
            ReactorError::TimerError => {
                write!(fmt, "error when scheduling timeout")
            },
        }
    }
}

impl ReactorConfig {
    /// Create a new default ReactorConfig.
    pub fn new() -> ReactorConfig {
        ReactorConfig::default()
    }

    /// Set the timer capacity.
    ///
    /// This is used by the event loop to specify the number of timers allowed. An indication that
    /// this should be increased if if a `ReactorError::TimerError` is returned.
    pub fn timer_capacity(mut self, cap: usize) -> ReactorConfig {
        self.timer_capacity = cap;
        self
    }

    /// Set the tick interval for the timer.
    pub fn timer_tick_interval_ms(mut self, ms: u64) -> ReactorConfig {
        self.timer_tick_interval_ms = Some(ms);
        self
    }

    fn to_event_loop_config(self) -> EventLoopConfig {
        let mut event_config = EventLoopConfig::new();
        event_config.timer_capacity(self.timer_capacity);
        match self.timer_tick_interval_ms {
            Some(ms) => { event_config.timer_tick_ms(ms); },
            None => {},
        }
        event_config
    }
}

impl Default for ReactorConfig {
    fn default() -> Self {
        ReactorConfig {
            timer_capacity: 1024,
            timer_tick_interval_ms: None,
        }
    }
}

impl<P: Protocol> Reactor<P> {
    /// Create a new Reactor with the default options.
    pub fn new(proto: P) -> io::Result<Reactor<P>> {
        Reactor::with_configuration(proto, ReactorConfig::default())
    }

    /// Create a new Reactor with the specified configuration.
    pub fn with_configuration(proto: P, config: ReactorConfig) -> io::Result<Reactor<P>> {
        let event_loop = try!(EventLoop::configured(config.to_event_loop_config()));
        let handler = ReactorHandler::new(proto);

        Ok(Reactor(event_loop, handler))
    }

    /// Start and run the Reactor.
    pub fn run(&mut self) -> io::Result<()> {
        panic!("unimplemented");
        // TODO: Implement this
    }

    /// Shutdown and consume the Reactor.
    pub fn shutdown(self) {
        panic!("unimplemented");
        // TODO: Implement this
    }

    /// spin_once the Reactor for a single iteration.
    ///
    /// This is mostly used for test purposes.
    pub fn spin_once(&mut self) -> io::Result<()> {
        let &mut Reactor(ref mut event_loop, ref mut handler) = self;
        event_loop.run_once(handler, Some(1000))
    }
}

#[cfg(test)]
mod tests {
    use test_helpers::{FakeProtocol, FakeSocket};
    use mio::{EventSet};
    use mio::unix::{pipe};
    use std::os::unix::io::{AsRawFd};
    use std::io::Write;
    use reactor::{Reactor, Configurer, ReactorConfig};

    #[test]
    fn test_reactor_read_write() {
        let (r, mut w) = pipe().unwrap();
        let buf = [1, 2, 3, 4];
        assert!(w.write(&buf).is_ok());

        let mut r = FakeSocket::PReader(r);
        let mut w = FakeSocket::PWriter(w);

        let read_fd = r.as_raw_fd();
        let write_fd = w.as_raw_fd();

        let mut proto = FakeProtocol::new();
        let mut reactor = Reactor::new(proto.clone()).unwrap();

        assert!(reactor.add_socket(&mut r, EventSet::readable()).is_ok());
        assert!(reactor.add_socket(&mut w, EventSet::writable()).is_ok());

        assert!(reactor.spin_once().is_ok());

        assert_eq!(proto.readable_fd(), Some(read_fd));
        assert_eq!(proto.writable_fd(), Some(write_fd));
        assert_eq!(proto.error_fd(), None);
    }

    #[test]
    fn test_reactor_periodic_read() {
        let (r, mut w) = pipe().unwrap();
        let buf = [1, 2, 3, 4];
        assert!(w.write(&buf).is_ok());

        let mut r = FakeSocket::PReader(r);

        let read_fd = r.as_raw_fd();

        let mut proto = FakeProtocol::new();
        let mut reactor = Reactor::new(proto.clone()).unwrap();

        assert!(reactor.add_socket(&mut r, EventSet::readable()).is_ok());

        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        proto.clear_all();
        assert!(w.write(&buf).is_ok());

        assert_eq!(proto.readable_fd(), None);
        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));
    }

    #[test]
    fn test_reactor_timeout() {
        let (r, _w) = pipe().unwrap();
        let (r2, mut w2) = pipe().unwrap();
        let buf = [1, 2, 3, 4];
        assert!(w2.write(&buf).is_ok());

        let mut r = FakeSocket::PReader(r);
        let mut r2 = FakeSocket::PReader(r2);

        let read_fd = r.as_raw_fd();
        let read2_fd = r2.as_raw_fd();

        let mut proto = FakeProtocol::new();
        let config = ReactorConfig::default().timer_tick_interval_ms(20);
        let mut reactor = Reactor::with_configuration(proto.clone(), config).unwrap();

        let res = reactor.add_socket_timeout(&mut r, EventSet::readable(), 40);
        assert!(res.is_ok());
        let token = res.unwrap();

        proto.add_socket(&mut r2, EventSet::readable());
        ::std::thread::sleep_ms(60);
        assert!(reactor.spin_once().is_ok());

        assert_eq!(proto.readable_fd(), None);
        assert_eq!(proto.timeout_fd(), Some(read_fd));
        proto.clear_all();

        let res = reactor.update_socket_timeout(token, EventSet::none(), 40);
        assert!(res.is_ok());

        ::std::thread::sleep_ms(60);
        assert!(reactor.spin_once().is_ok());

        assert_eq!(proto.readable_fd(), Some(read2_fd));
        assert_eq!(proto.timeout_fd(), Some(read_fd));
   }

    #[test]
    fn test_reactor_disconnect() {
        let (r, mut w) = pipe().unwrap();
        let buf = [1, 2, 3, 4];
        assert!(w.write(&buf).is_ok());

        let mut r = FakeSocket::PReader(r);
        let read_fd = r.as_raw_fd();

        let mut proto = FakeProtocol::new();
        let mut reactor = Reactor::new(proto.clone()).unwrap();

        assert!(reactor.add_socket(&mut r, EventSet::readable()).is_ok());

        drop(w); // Trigger disconnect by closing writer

        assert!(reactor.spin_once().is_ok());

        assert_eq!(proto.readable_fd(), Some(read_fd));
        assert_eq!(proto.disconnect_fd(), Some(read_fd));
    }

    #[test]
    fn test_reactor_update_socket() {
        let (r, mut w) = pipe().unwrap();
        let buf = [1, 2];
        assert!(w.write(&buf).is_ok());

        let mut r = FakeSocket::PReader(r);

        let read_fd = r.as_raw_fd();

        let mut proto = FakeProtocol::new();
        let mut reactor = Reactor::new(proto.clone()).unwrap();

        let res = reactor.add_socket(&mut r, EventSet::readable());
        assert!(res.is_ok());
        let token = res.unwrap();

        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        assert!(reactor.update_socket(token, EventSet::none()).is_ok());

        proto.clear_all();
        assert_eq!(proto.readable_fd(), None);

        assert!(w.write(&buf).is_ok());
        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), None);
    }

    #[test]
    fn test_reactor_protocol_socket_configuration() {
        let (r, mut w) = pipe().unwrap();
        let buf = [1, 2];
        assert!(w.write(&buf).is_ok());
        assert!(w.flush().is_ok());
        let mut r = FakeSocket::PReader(r);
        let read_fd = r.as_raw_fd();

        let (_r2, mut w2) = pipe().unwrap();
        let write_fd = w2.as_raw_fd();
        assert!(w2.write(&buf).is_ok());
        assert!(w2.flush().is_ok());
        let mut w2 = FakeSocket::PWriter(w2);

        let mut proto = FakeProtocol::new();
        let mut reactor = Reactor::new(proto.clone()).unwrap();

        let res = reactor.add_socket(&mut r, EventSet::readable());
        assert!(res.is_ok());
        let read_token = res.unwrap();

        // Add writer and update reader to none
        proto.add_socket(&mut w2, EventSet::writable());
        proto.update_socket(read_token, EventSet::none());

        // spin_once: Writer is not writeable yet, reader is readable from first
        // add_socket call
        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        proto.clear_all();
        assert_eq!(proto.readable_fd(), None);

        // spin_once: reader was updated to none events, spin_once and assert not readable
        // Writer should now be writeable
        assert!(w.write(&buf).is_ok());
        // Set back to readable for next spin_once
        proto.update_socket(read_token, EventSet::readable());
        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), None);
        assert_eq!(proto.writable_fd(), Some(write_fd));

        // spin_once: Set back to readable again
        assert!(w.write(&buf).is_ok());
        // spin_once: Remove read socket for next spin_once
        proto.remove_socket(read_token);
        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        proto.clear_all();
        assert_eq!(proto.readable_fd(), None);

        // spin_once: Removed read socket, assert no readable sockets
        assert!(w.write(&buf).is_ok());
        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), None);
    }

    #[test]
    fn test_reactor_remove_socket() {
        let (r, mut w) = pipe().unwrap();
        let buf = [1, 2, 3, 4];
        assert!(w.write(&buf).is_ok());

        let mut r = FakeSocket::PReader(r);

        let read_fd = r.as_raw_fd();

        let mut proto = FakeProtocol::new();
        let mut reactor = Reactor::new(proto.clone()).unwrap();

        let res = reactor.add_socket(&mut r, EventSet::readable());
        assert!(res.is_ok());
        let token = res.unwrap();

        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        assert!(reactor.remove_socket(token).is_ok());

        proto.clear_all();
        assert!(w.write(&buf).is_ok());

        assert_eq!(proto.readable_fd(), None);
        assert!(reactor.spin_once().is_ok());
        assert_eq!(proto.readable_fd(), None);
    }
}
