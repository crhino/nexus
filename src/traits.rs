use future::{Future, Promise};

/// Owns the socket
pub trait Transport {
    type Buffer;

    /// Returns a buffer object that will write data to the underlying socket. This is used by the
    /// Codecs in order to efficiently write data without copying.
    fn buffer(&mut self) -> &mut Self::Buffer;
    fn spawned(&mut self);
    fn close(&mut self);
    fn transport_closed(&mut self);
    fn read(&mut self) -> &[u8];
    /// Tells transport that "bytes" number of bytes have been read
    fn consume(&mut self, bytes: usize);
    /// Called when socket changes state to being writable.
    fn writable(&mut self);
}

pub trait Codec<B> {
    type Input;
    type Output;

    /// Codec should write encoded data to buffer and finish the promise.
    fn encode(&mut self, buffer: &mut B, input: Self::Input, promise: Promise<()>);
    // If decode returns None that means the Codec needs more data, otherwise it returns a tuple of
    // the number of bytes used and an Output object.
    fn decode(&mut self, buffer: &[u8]) -> Option<(usize, Self::Output)>;
}

pub trait Protocol {
    type Output;
    type Input;

    /// Does not currently respect ctx.close()
    fn spawned<C>(&mut self, ctx: &mut C) where C: Context;
    fn closed<C>(&mut self, ctx: &mut C) where C: Context;
    fn received_data<C>(&mut self, ctx: &mut C, data: Self::Input) where C: Context<Write=Self::Output>;
    /// Called when socket changes state to being writable.
    fn writable<C>(&mut self, ctx: &mut C) where C: Context<Write=Self::Output>;
}

pub trait Context {
    type Write;

    /// The write method can only be called once per stage. The object will be returned if
    /// the object was not scheduled to be written.
    fn write(&mut self, obj: Self::Write) -> Result<Future<()>, Self::Write>;
    fn close(&mut self);
}
