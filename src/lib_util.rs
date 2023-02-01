pub struct PointerWrapper<T> {
    ptr: *mut T,
}

unsafe impl<T> Send for PointerWrapper<T> {}

unsafe impl<T> Sync for PointerWrapper<T> {}

impl<T> Clone for PointerWrapper<T> {
    fn clone(&self) -> Self {
        PointerWrapper { ptr: self.ptr }
    }
}

impl<T> PointerWrapper<T> {
    pub fn new(ptr: *mut T) -> Self {
        PointerWrapper { ptr }
    }

    pub fn get(&self) -> *mut T {
        self.ptr
    }
}
