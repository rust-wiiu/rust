macro_rules! imports_section {
    // Main entry point
    ($name:literal, [$($func:literal),*], [$($data:literal),*]) => {
        imports_section!(@fimport $name, $($func),*);
        imports_section!(@dimport $name, $($data),*);
    };

    // Function imports - non-empty
    (@fimport $name:literal, $($func:literal),+) => {
        $crate::arch::global_asm!(
            concat!(".section .fimport_", $name, ", \"ax\""),
            ".align 4",
            ".long 1",
            ".long 0x00000000",
            concat!(".string \"", $name, "\""),
            ".balign 8, 0",
        );
        $(
            $crate::arch::global_asm!(
                concat!(".section .fimport_", $name, ".", $func, ", \"ax\""),
                concat!(".global ", $func),
                concat!(".type ", $func, ", @function"),
                concat!($func, ":"),
                ".long 0x0",
                ".long 0x0",
            );
        )*
    };
    (@fimport $name:literal,) => {};

    // Data imports - non-empty
    (@dimport $name:literal, $($data:literal),+) => {
        $crate::arch::global_asm!(
            concat!(".section .dimport_", $name, ", \"a\""),
            ".align 4",
            ".long 1",
            ".long 0x00000000",
            concat!(".string \"", $name, "\""),
            ".balign 8, 0",
        );
        $(
            $crate::arch::global_asm!(
                concat!(".section .dimport_", $name, ".", $data, ", \"a\""),
                concat!(".global ", $data),
                concat!(".type ", $data, ", @object"),
                concat!($data, ":"),
                ".long 0x0",
                ".long 0x0",
            );
        )*
    };
    (@dimport $name:literal,) => {};
}

imports_section!(
    "coreinit",
    [
        "exit",
        "OSGetSystemTime",
        "OSSleepTicks",
        "OSGetThreadSpecific",
        "OSSetThreadSpecific",
        "OSReport",
        "OSConsoleWrite",
        "OSGetSystemInfo",
        "OSGetTime",
        "OSYieldThread",
        "OSGetCoreCount"
    ],
    ["MEMAllocFromDefaultHeapEx", "MEMFreeToDefaultHeap"]
);

use crate::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn exit(code: i32) -> !;
    pub fn OSGetSystemTime() -> i64;
    pub fn OSSleepTicks(ticks: i64);
    pub fn OSGetThreadSpecific(index: i32) -> *mut c_void;
    pub fn OSSetThreadSpecific(index: i32, ptr: *mut c_void);

    pub static MEMAllocFromDefaultHeapEx:
        unsafe extern "C" fn(size: u32, align: i32) -> *mut c_void;
    pub static MEMFreeToDefaultHeap: unsafe extern "C" fn(ptr: *mut c_void);

    // pub unsafe fn OSReport(msg: *const c_char);
    pub unsafe fn OSConsoleWrite(msg: *const c_char, size: u32);

    pub fn OSGetSystemInfo() -> *const OSSystemInfo;
    /// Ticks since 2000-01-01 00:00.
    pub fn OSGetTime() -> i64;

    pub fn OSYieldThread();
    pub fn OSGetCoreCount() -> u32;
}

#[repr(C)]
pub struct OSSystemInfo {
    pub bus_clock_speed: u32,
    pub core_clock_speed: u32,
    /// Ticks from 2000-01-01 until system reset.
    pub time_base: i64,
    pub l2_size: [u32; 3],
    pub cpu_freq_ratio: u32,
}
