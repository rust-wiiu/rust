use crate::ffi::OsString;
use crate::path::PathBuf;
use crate::sync::OnceLock;
use crate::sys::FromInner;
use crate::sys::os_str::Buf;
use crate::sys::pal::abi;

/// Returns the app's TEMP directory, which the system deletes when the app exits.
pub fn temp_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        // One spare byte, so the path is always nul-terminated.
        let mut buf = [0u8; abi::FS_MAX_FULLPATH_SIZE + 1];
        let status = unsafe {
            abi::TEMPInit();
            let mut id = 0;
            let status = abi::TEMPCreateAndInitTempDir(
                u32::MAX,
                abi::TEMP_DEVICE_PREFERENCE_FREE_SPACE,
                &mut id,
            );
            if status != 0 {
                status
            } else {
                abi::TEMPGetDirGlobalPath(
                    id,
                    buf.as_mut_ptr().cast(),
                    abi::FS_MAX_FULLPATH_SIZE as u32,
                )
            }
        };
        if status != 0 {
            panic!("failed to create the TEMP directory (status {status})");
        }
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        PathBuf::from(OsString::from_inner(Buf { inner: buf[..len].to_vec() }))
    })
    .clone()
}
