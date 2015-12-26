use std::sync::{Arc, Mutex};
use std::os::unix::io::{RawFd, AsRawFd};
use mio::{Selector, PollOpt, Token, EventSet, Evented};
use mio::unix::{PipeReader, PipeWriter};
use std::io::{self, Read};

use reactor::{ReactorError, Configurer};
use protocol::{Protocol};

#[derive(Clone)]
pub struct FakeProtocol<'a> {
    inner: Arc<Inner<'a>>,
}

struct Inner<'a> {
    on_readable_fd: Mutex<Option<RawFd>>,
    on_writable_fd: Mutex<Option<RawFd>>,
    on_timeout_fd: Mutex<Option<RawFd>>,
    on_disconnect_fd: Mutex<Option<RawFd>>,
    on_error_fd: Mutex<Option<RawFd>>,
    additions: Mutex<Vec<(&'a mut FakeSocket, EventSet)>>,
    updates: Mutex<Vec<(Token, EventSet)>>,
    removes: Mutex<Vec<Token>>,
}

#[derive(Debug)]
pub enum FakeSocket {
    PReader(PipeReader),
    PWriter(PipeWriter),
}
use test_helpers::FakeSocket::*;

impl<'a> Evented for &'a mut FakeSocket {
    fn register(&self, selector:
                &mut Selector,
                token: Token,
                interest: EventSet,
                opts: PollOpt) -> io::Result<()> {
        match **self {
            PReader(ref r) => r.register(selector, token, interest, opts),
            PWriter(ref w) => w.register(selector, token, interest, opts),
        }
    }

    fn reregister(&self,
                  selector: &mut Selector,
                  token: Token,
                  interest: EventSet,
                  opts: PollOpt) -> io::Result<()> {
        match **self {
            PReader(ref r) => r.reregister(selector, token, interest, opts),
            PWriter(ref w) => w.reregister(selector, token, interest, opts),
        }
    }

    fn deregister(&self, selector: &mut Selector) -> io::Result<()> {
        match **self {
            PReader(ref r) => r.deregister(selector),
            PWriter(ref w) => w.deregister(selector),
        }
    }
}

impl AsRawFd for FakeSocket {
    fn as_raw_fd(&self) -> RawFd {
        match *self {
            PReader(ref r) => r.as_raw_fd(),
            PWriter(ref w) => w.as_raw_fd(),
        }
    }
}


impl<'a> Protocol for FakeProtocol<'a> {
    type Socket = &'a mut FakeSocket;

    fn on_readable(&mut self, configurer: &mut Configurer<Self>, socket: &mut Self::Socket, _token: Token) {
        let mut buf = [0u8, 32];
        let res = match *socket {
            &mut PReader(ref mut r) => r.read(&mut buf),
            &mut PWriter(_) => panic!("A readable writer was found."),
        };

        match res {
            Ok(_) => { },
            Err(e) => panic!("{:?}", e),
        }
        {
            let mut guard = self.inner.on_readable_fd.lock().unwrap();
            *guard = Some(socket.as_raw_fd());
        }
        self.configure_sockets(configurer);
    }

    fn on_writable(&mut self, configurer: &mut Configurer<Self>, socket: &mut Self::Socket, _token: Token) {
        {
            let mut guard = self.inner.on_writable_fd.lock().unwrap();
            *guard = Some(socket.as_raw_fd());
        }
        self.configure_sockets(configurer);
    }

    fn on_timeout(&mut self, configurer: &mut Configurer<Self>, socket: &mut Self::Socket, _token: Token) {
        {
            let mut guard = self.inner.on_timeout_fd.lock().unwrap();
            *guard = Some(socket.as_raw_fd());
        }
        self.configure_sockets(configurer);
    }

    fn on_disconnect(&mut self, configurer: &mut Configurer<Self>, socket: &mut Self::Socket, _token: Token) {
        {
            let mut guard = self.inner.on_disconnect_fd.lock().unwrap();
            *guard = Some(socket.as_raw_fd());
        }
        self.configure_sockets(configurer);
    }

    fn on_socket_error(&mut self, configurer: &mut Configurer<Self>, socket: &mut Self::Socket, _token: Token) {
        {
            let mut guard = self.inner.on_error_fd.lock().unwrap();
            *guard = Some(socket.as_raw_fd());
        }
        self.configure_sockets(configurer);
    }

    fn on_event_loop_error(&mut self, error: ReactorError<Self::Socket>) {
        panic!("Received error: {:?}", error);
    }
}

impl<'a> FakeProtocol<'a> {
    pub fn new() -> FakeProtocol<'a> {
        let inner = Inner{
            on_readable_fd: Mutex::new(None),
            on_writable_fd: Mutex::new(None),
            on_timeout_fd: Mutex::new(None),
            on_disconnect_fd: Mutex::new(None),
            on_error_fd: Mutex::new(None),
            // phantom: PhantomData,
            additions: Mutex::new(Vec::new()),
            updates: Mutex::new(Vec::new()),
            removes: Mutex::new(Vec::new()),
        };

        FakeProtocol{
            inner: Arc::new(inner),
        }
    }

    pub fn clear_all(&mut self) {
        let mut guard = self.inner.on_error_fd.lock().unwrap();
        *guard = None;

        let mut guard = self.inner.on_readable_fd.lock().unwrap();
        *guard = None;

        let mut guard = self.inner.on_writable_fd.lock().unwrap();
        *guard = None;

        let mut guard = self.inner.on_timeout_fd.lock().unwrap();
        *guard = None;

        let mut guard = self.inner.on_disconnect_fd.lock().unwrap();
        *guard = None;
    }

    pub fn add_socket(&mut self, socket: &'a mut FakeSocket, events: EventSet) {
        let mut guard = self.inner.additions.lock().unwrap();
        guard.push((socket, events));
    }

    pub fn update_socket(&mut self, socket: Token, events: EventSet) {
        let mut guard = self.inner.updates.lock().unwrap();
        guard.push((socket, events));
    }

    pub fn remove_socket(&mut self, socket: Token) {
        let mut guard = self.inner.removes.lock().unwrap();
        guard.push(socket);
    }

    fn configure_sockets(&mut self, event_configurer: &mut Configurer<Self>) {
        let mut guard = self.inner.additions.lock().unwrap();
        while let Some((socket, events)) = guard.pop() {
            event_configurer.add_socket(socket, events);
        }

        let mut guard = self.inner.updates.lock().unwrap();
        while let Some((token, events)) = guard.pop() {
            event_configurer.update_socket(token, events);
        }

        let mut guard = self.inner.removes.lock().unwrap();
        while let Some(token) = guard.pop() {
            event_configurer.remove_socket(token);
        }
    }

    pub fn readable_fd(&mut self) -> Option<RawFd> {
        let guard = self.inner.on_readable_fd.lock().unwrap();
        (*guard).clone()
    }

    pub fn writable_fd(&mut self) -> Option<RawFd> {
        let guard = self.inner.on_writable_fd.lock().unwrap();
        (*guard).clone()
    }

    pub fn timeout_fd(&mut self) -> Option<RawFd> {
        let guard = self.inner.on_timeout_fd.lock().unwrap();
        (*guard).clone()
    }

    pub fn disconnect_fd(&mut self) -> Option<RawFd> {
        let guard = self.inner.on_readable_fd.lock().unwrap();
        (*guard).clone()
    }

    pub fn error_fd(&mut self) -> Option<RawFd> {
        let guard = self.inner.on_error_fd.lock().unwrap();
        (*guard).clone()
    }
}
