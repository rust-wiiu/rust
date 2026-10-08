// #[expect(dead_code)]
#[path = "../unsupported/common.rs"]
mod unsupported_common;

pub use unsupported_common::{abort_internal, cleanup, init, unsupported};

pub mod abi;
pub mod ticks;

#[cfg(not(test))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".crt0")]
unsafe extern "C" fn __rpx_start(argc: i32, argv: *const *const u8) -> ! {
    unsafe extern "C" {
        fn main(argc: i32, argv: *const *const u8) -> i32;
    }

    unsafe { abi::exit(main(argc, argv)) }
}
