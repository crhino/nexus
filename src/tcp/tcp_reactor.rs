use reactor::{Reactor};
use protocol::{Protocol};
use mio::tcp::{TcpListener};
use mio::{Evented};

pub struct TcpReactor<P: Protocol> {
    reactor: Reactor<P>,
}
