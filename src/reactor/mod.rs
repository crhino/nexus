pub mod configurer;
mod handler;

pub use reactor::configurer::{Configurer};
pub use mio::Token;
pub use reactor::handler::ReactorHandler;

use reactor::configurer::{ProtocolConfigurer};

use mio::{Evented, EventLoop, EventLoopConfig, PollOpt, EventSet, Handler, Timeout};
use mio::util::{Slab};
use std::io::{self};
use std::error::Error;
use std::fmt;

use protocol::Protocol;

const SLAB_GROW_SIZE: usize = 1024;

/// A Reactor runs the event loop and manages sockets
pub struct Reactor<P: Protocol>(EventLoop<ReactorHandler<P>>, ReactorHandler<P>);

pub struct ReactorConfig {
    timer_capacity: usize,
    timer_tick_interval_ms: Option<u64>,
}

#[derive(Debug)]
pub enum ReactorError<S> {
    IoError(io::Error, S),
    NoSocketFound(Token),
    TimerError,
}

impl<S> fmt::Display for ReactorError<S> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReactorError::IoError(ref e, ref s) => {
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

// impl<S: fmt::Debug> Error for ReactorError<S> {
//     fn description(&self) -> &str {
//         match *self {
//             ReactorError::MaxConnectionsReached(ref s) => {
//                 "max connections reached"
//             },
//             ReactorError::IoError((ref e, ref s)) => {
//                 format!("io error: {}", e.description())
//             },
//         }
//     }
// }

impl ReactorConfig {
    pub fn new() -> ReactorConfig {
        ReactorConfig::default()
    }

    pub fn timer_capacity(mut self, cap: usize) -> ReactorConfig {
        self.timer_capacity = cap;
        self
    }

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
    pub fn new(proto: P) -> io::Result<Reactor<P>> {
        Reactor::with_configuration(proto, ReactorConfig::default())
    }

    pub fn with_configuration(proto: P, config: ReactorConfig) -> io::Result<Reactor<P>> {
        let event_loop = try!(EventLoop::configured(config.to_event_loop_config()));
        let handler = ReactorHandler::new(proto);

        Ok(Reactor(event_loop, handler))
    }

    pub fn start(&mut self) {
    }

    pub fn shutdown(self) {
    }

    pub fn tick(&mut self) -> io::Result<()> {
        let &mut Reactor(ref mut event_loop, ref mut handler) = self;
        event_loop.run_once(handler, Some(1000))
    }
}

#[cfg(test)]
mod tests {
    use test_helpers::{FakeProtocol, FakeSocket};
    use mio::{Evented, EventSet};
    use mio::unix::{pipe};
    use std::os::unix::io::{AsRawFd};
    use std::io::Write;
    use reactor::{Reactor, Configurer};

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

        assert!(reactor.tick().is_ok());

        assert_eq!(proto.readable_fd(), Some(read_fd));
        assert_eq!(proto.writable_fd(), Some(write_fd));
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

        assert!(reactor.tick().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        proto.clear_all();
        assert!(w.write(&buf).is_ok());

        assert_eq!(proto.readable_fd(), None);
        assert!(reactor.tick().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));
    }

    #[test]
    fn test_reactor_timeout() {
        let (r, w) = pipe().unwrap();

        let mut r = FakeSocket::PReader(r);

        let read_fd = r.as_raw_fd();

        let mut proto = FakeProtocol::new();
        let mut reactor = Reactor::new(proto.clone()).unwrap();

        let res = reactor.add_socket_timeout(&mut r, EventSet::readable(), 1);
        assert!(res.is_ok());
        let token = res.unwrap();

        ::std::thread::sleep_ms(1);
        assert!(reactor.tick().is_ok());

        assert_eq!(proto.readable_fd(), None);
        assert_eq!(proto.timeout_fd(), Some(read_fd));
        proto.clear_all();

        let res = reactor.update_socket_timeout(token, EventSet::none(), 1);
        assert!(res.is_ok());

        ::std::thread::sleep_ms(1);
        assert!(reactor.tick().is_ok());

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

        assert!(reactor.tick().is_ok());

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

        assert!(reactor.tick().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        assert!(reactor.update_socket(token, EventSet::none()).is_ok());

        proto.clear_all();
        assert_eq!(proto.readable_fd(), None);

        assert!(w.write(&buf).is_ok());
        assert!(reactor.tick().is_ok());
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

        let (r2, mut w2) = pipe().unwrap();
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

        // Tick: Writer is not writeable yet, reader is readable from first
        // add_socket call
        assert!(reactor.tick().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        proto.clear_all();
        assert_eq!(proto.readable_fd(), None);

        // Tick: reader was updated to none events, tick and assert not readable
        // Writer should now be writeable
        assert!(w.write(&buf).is_ok());
        // Set back to readable for next tick
        proto.update_socket(read_token, EventSet::readable());
        assert!(reactor.tick().is_ok());
        assert_eq!(proto.readable_fd(), None);
        assert_eq!(proto.writable_fd(), Some(write_fd));

        // Tick: Set back to readable again
        assert!(w.write(&buf).is_ok());
        // Tick: Remove read socket for next tick
        proto.remove_socket(read_token);
        assert!(reactor.tick().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        proto.clear_all();
        assert_eq!(proto.readable_fd(), None);

        // Tick: Removed read socket, assert no readable sockets
        assert!(w.write(&buf).is_ok());
        assert!(reactor.tick().is_ok());
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

        assert!(reactor.tick().is_ok());
        assert_eq!(proto.readable_fd(), Some(read_fd));

        assert!(reactor.remove_socket(token).is_ok());

        proto.clear_all();
        assert!(w.write(&buf).is_ok());

        assert_eq!(proto.readable_fd(), None);
        assert!(reactor.tick().is_ok());
        assert_eq!(proto.readable_fd(), None);
    }
}
