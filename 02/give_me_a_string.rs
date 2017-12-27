use std::ffi::CString;
use std::os::raw::c_char;

#[no_mangle]
pub fn give_me_a_string() -> *const c_char {
    CString::new("Hello world")
        .unwrap()
        .into_raw()
}
