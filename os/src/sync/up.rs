use core::cell::{RefCell, RefMut};
pub struct UPsafeCell<T>{
    pub inner:RefCell<T>
}
unsafe impl<T> Sync for UPsafeCell<T>{}

impl<T> UPsafeCell<T>{
    pub fn new(value: T)->UPsafeCell<T>{
        Self{
            inner: RefCell::new(value),
        }
    }
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}