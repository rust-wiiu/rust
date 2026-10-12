use cafe_sys::coreinit::launch;

pub use super::common::Args;
use crate::ffi::{CStr, OsString};
use crate::sys::FromInner;
use crate::sys::os_str::Buf;
use crate::{ptr, slice};

pub fn args() -> Args {
    let mut argc = 0;
    let mut argv = ptr::null();
    unsafe { launch::get_argc_argv(&mut argc, &mut argv) };
    if argv.is_null() {
        return Args::new(Vec::new());
    }
    let argv = unsafe { slice::from_raw_parts(argv, usize::try_from(argc).unwrap_or(0)) };
    let args = argv
        .iter()
        .filter(|arg| !arg.is_null())
        .map(|&arg| {
            let bytes = unsafe { CStr::from_ptr(arg) }.to_bytes();
            OsString::from_inner(Buf { inner: bytes.to_vec() })
        })
        .collect();
    Args::new(args)
}
