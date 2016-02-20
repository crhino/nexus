use std::io;
use std::sync::{Arc, Mutex, Condvar};
use std::fmt;

struct Inner<T> {
    data: Mutex<Option<io::Result<T>>>,
    cond: Condvar,
}

impl<T: fmt::Debug> fmt::Debug for Inner<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("Inner")
            .finish()
    }
}

impl<T> Inner<T> {
    fn new() -> Inner<T> {
        Inner{
            data: Mutex::new(None),
            cond: Condvar::new(),
        }
    }

    fn get(&self) -> io::Result<T> {
        let mut guard = self.data.lock().expect("lock poisoned");
        while guard.is_none() {
            guard = self.cond.wait(guard).expect("lock posioned while waiting");
        }

        // We know this is a Some variant
        guard.take().unwrap()
    }

    fn set(&self, data: io::Result<T>) {
        let mut guard = self.data.lock().expect("lock poisoned");
        *guard = Some(data);
        self.cond.notify_one();
    }

    fn is_done(&self) -> bool {
        let guard = self.data.lock().expect("lock poisoned");
        guard.is_some()
    }
}

#[derive(Debug)]
pub struct NexusFuture<T> {
    inner: Arc<Inner<T>>,
}

impl<T> NexusFuture<T> {
    pub fn get(&self) -> io::Result<T> {
        self.inner.get()
    }

    pub fn is_done(&self) -> bool {
        self.inner.is_done()
    }
}

pub struct NexusPromise<T> {
    inner: Arc<Inner<T>>,
}

impl<T> NexusPromise<T> {
    pub fn set(&self, data: io::Result<T>) {
        self.inner.set(data)
    }
}

pub fn pair<T>() -> (NexusPromise<T>, NexusFuture<T>) {
    let inner = Arc::new(Inner::new());
    let promise = NexusPromise {
        inner: inner.clone(),
    };

    let future = NexusFuture {
        inner: inner,
    };

    (promise, future)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrous::dsl::*;
    use std::thread;
    use std::sync::mpsc::channel;
    use std::io::{self, ErrorKind};

    #[test]
    fn test_future_get_promise_set() {
        let (promise, future) = pair::<u8>();

        let handle = thread::spawn(move || {
            promise.set(Ok(27u8));
        });
        handle.join().unwrap();

        let res = future.get();
        expect(&res).to(be_ok());
        expect(&res.unwrap()).to(equal(&27));
    }

    #[test]
    fn test_future_get_promise_fail() {
        let (promise, future) = pair::<u8>();

        let handle = thread::spawn(move || {
            promise.set(Err(io::Error::new(ErrorKind::Other, "boom!")));
        });
        handle.join().unwrap();

        let res = future.get();
        expect(&res).to(be_err());
        expect(&res.unwrap_err().kind()).to(equal(&ErrorKind::Other));
    }

    #[test]
    fn test_future_is_done() {
        let (promise, future) = pair::<u8>();

        expect(&future.is_done()).to(equal(&false));
        let handle = thread::spawn(move || {
            promise.set(Ok(27u8));
        });
        handle.join().unwrap();

        expect(&future.is_done()).to(equal(&true));
    }
}
