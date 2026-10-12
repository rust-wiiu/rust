use cafe_sys::coreinit::fs::MAX_PATH;
use cafe_sys::nn::temp;

use crate::ffi::OsString;
use crate::path::PathBuf;
use crate::sync::OnceLock;
use crate::sys::FromInner;
use crate::sys::os_str::Buf;

/// Returns the app's TEMP directory, which the system deletes when the app exits.
pub fn temp_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        // One spare byte, so the path is always nul-terminated.
        let mut buf = [0u8; MAX_PATH + 1];
        let status = unsafe {
            temp::init();
            let mut id = temp::DirId::INVALID;
            let status = temp::create_and_init_temp_dir(
                u32::MAX,
                temp::DevicePreference::FreeSpace,
                &mut id,
            );
            if status != temp::Status::OK {
                status
            } else {
                temp::get_dir_global_path(id, buf.as_mut_ptr().cast(), MAX_PATH as u32)
            }
        };
        if status != temp::Status::OK {
            panic!("failed to create the TEMP directory (status {})", status.0);
        }
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        PathBuf::from(OsString::from_inner(Buf { inner: buf[..len].to_vec() }))
    })
    .clone()
}
