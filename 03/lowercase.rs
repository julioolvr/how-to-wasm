use std::mem;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub fn lowercase(data: *const c_char) -> *const c_char {
    let incoming_str;

    unsafe {
        incoming_str = CStr::from_ptr(data).to_str().unwrap().to_owned();
    }

    let lowercased = CString::new(incoming_str.to_lowercase()).unwrap();
    lowercased.into_raw()
}

#[no_mangle]
pub fn alloc(size: usize) -> *const c_void {
    let buf = Vec::with_capacity(size);
    let ptr = buf.as_ptr();
    mem::forget(buf);
    ptr
}
