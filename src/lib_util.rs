use std::os::raw::c_char;

pub fn string_from_lengthen_ptr(ptr: *const c_char, len: u32) -> String {
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    String::from_utf8_lossy(slice).to_string()
}
