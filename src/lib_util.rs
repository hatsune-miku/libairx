use std::ffi::c_char;

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

// ---

pub fn string_from_lengthed_ptr(ptr: *const c_char, len: u32) -> String {
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    String::from_utf8_lossy(slice).to_string()
}
