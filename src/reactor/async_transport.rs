use rotor::{Response, Scope, Machine, EventSet, PollOpt};

pub struct AsyncTransport<S> {
    socket: S,
}

impl<S> Machine for AsyncTransport<S> {
    type Context = ();
    type Seed = Void;

    fn create(void: Self::Seed, scope: &mut Scope<Self::Context>) -> Result<Self, Box<Error>> {
        unreachable(void);
    }

    fn ready(self, events: EventSet, scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
    }

    fn spawned(self, scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
    }

    fn timeout(self, scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
    }

    fn wakeup(self, scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
    }

    // fn spawn_error(self, _scope: &mut Scope<Self::Context>, error: Box<Error>) -> Option<Self> {}
}

#[cfg(test)]
mod tests {
}
