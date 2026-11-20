pub trait Observer<E>: Send + Sync {
    fn observe(&self, event: &E);
}

impl<E, F: Fn(&E) + Send + Sync> Observer<E> for F {
    fn observe(&self, event: &E) {
        self(event)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Silent;

impl<E> Observer<E> for Silent {
    fn observe(&self, _event: &E) {}
}
