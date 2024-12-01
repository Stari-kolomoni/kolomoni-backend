use std::sync::Arc;

use parking_lot::{ArcRwLockReadGuard, ArcRwLockWriteGuard, RawRwLock, RwLock};

pub struct Shared<D> {
    inner: Arc<RwLock<D>>,
}

impl<D> Shared<D> {
    pub fn new(data: D) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    pub fn read(&self) -> ArcRwLockReadGuard<RawRwLock, D> {
        RwLock::read_arc(&self.inner)
    }

    pub fn write(&self) -> ArcRwLockWriteGuard<RawRwLock, D> {
        RwLock::write_arc(&self.inner)
    }

    pub fn update<C>(&self, modification_closure: C)
    where
        C: FnOnce(&mut D),
    {
        let mut write_lock = self.write();

        modification_closure(&mut write_lock);
    }
}

impl<D> Clone for Shared<D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
