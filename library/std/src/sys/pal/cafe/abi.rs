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
        "OSGetCoreCount",
        "OSCreateThread",
        "OSResumeThread",
        "OSJoinThread",
        "OSDetachThread",
        "OSGetCurrentThread",
        "OSGetThreadPriority",
        "OSSetThreadDeallocator",
        "OSSetThreadName",
        "OSGetThreadName",
        "OSFastMutex_Lock",
        "OSFastMutex_Unlock",
        "OSInitEvent",
        "OSWaitEvent",
        "OSWaitEventWithTimeout",
        "OSSignalEvent",
        "FSInit",
        "FSAddClient",
        "FSInitCmdBlock",
        "FSGetMountSource",
        "FSMount",
        "FSOpenFile",
        "FSCloseFile",
        "FSReadFile",
        "FSWriteFile",
        "FSTruncateFile",
        "FSFlushFile",
        "FSGetPosFile",
        "FSSetPosFile",
        "FSGetStatFile",
        "FSGetStat",
        "FSOpenDir",
        "FSReadDir",
        "FSCloseDir",
        "FSMakeDir",
        "FSRemove",
        "FSRename",
        "FSChangeDir",
        "FSGetCwd"
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

    pub fn OSCreateThread(
        thread: *mut OSThread,
        entry: OSThreadProc,
        int_arg: i32,
        ptr_arg: *mut c_void,
        stack_top: *mut c_void,
        stack_size: u32,
        priority: i32,
        attr: u16,
    ) -> i32;
    pub fn OSResumeThread(thread: *mut OSThread) -> i32;
    pub fn OSJoinThread(thread: *mut OSThread, result: *mut i32) -> i32;
    pub fn OSDetachThread(thread: *mut OSThread);
    pub fn OSGetCurrentThread() -> *mut OSThread;
    pub fn OSGetThreadPriority(thread: *mut OSThread) -> i32;
    pub fn OSSetThreadDeallocator(
        thread: *mut OSThread,
        dealloc: OSThreadDeallocator,
    ) -> Option<OSThreadDeallocator>;
    pub fn OSSetThreadName(thread: *mut OSThread, name: *const c_char);
    pub fn OSGetThreadName(thread: *const OSThread) -> *const c_char;

    pub fn OSFastMutex_Lock(mutex: *mut OSFastMutex);
    pub fn OSFastMutex_Unlock(mutex: *mut OSFastMutex);
    pub fn OSInitEvent(event: *mut OSEvent, initial_state: i32, mode: i32);
    pub fn OSWaitEvent(event: *mut OSEvent);
    pub fn OSWaitEventWithTimeout(event: *mut OSEvent, timeout_ns: i64) -> i32;
    pub fn OSSignalEvent(event: *mut OSEvent);

    pub fn FSInit();
    pub fn FSAddClient(client: *mut FSClient, err_mask: u32) -> i32;
    pub fn FSInitCmdBlock(block: *mut FSCmdBlock);
    pub fn FSGetMountSource(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        source_type: u32,
        source: *mut FSMountSource,
        err_mask: u32,
    ) -> i32;
    pub fn FSMount(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        source: *mut FSMountSource,
        target: *mut c_char,
        bytes: u32,
        err_mask: u32,
    ) -> i32;
    pub fn FSOpenFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        path: *const c_char,
        mode: *const c_char,
        handle: *mut i32,
        err_mask: u32,
    ) -> i32;
    pub fn FSCloseFile(client: *mut FSClient, block: *mut FSCmdBlock, handle: i32, err_mask: u32) -> i32;
    pub fn FSReadFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        dest: *mut c_void,
        size: u32,
        count: u32,
        handle: i32,
        flag: u32,
        err_mask: u32,
    ) -> i32;
    pub fn FSWriteFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        source: *const c_void,
        size: u32,
        count: u32,
        handle: i32,
        flag: u32,
        err_mask: u32,
    ) -> i32;
    pub fn FSTruncateFile(client: *mut FSClient, block: *mut FSCmdBlock, handle: i32, err_mask: u32) -> i32;
    pub fn FSFlushFile(client: *mut FSClient, block: *mut FSCmdBlock, handle: i32, err_mask: u32) -> i32;
    pub fn FSGetPosFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        pos: *mut u32,
        err_mask: u32,
    ) -> i32;
    pub fn FSSetPosFile(client: *mut FSClient, block: *mut FSCmdBlock, handle: i32, pos: u32, err_mask: u32) -> i32;
    pub fn FSGetStatFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        stat: *mut FSStat,
        err_mask: u32,
    ) -> i32;
    pub fn FSGetStat(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        path: *const c_char,
        stat: *mut FSStat,
        err_mask: u32,
    ) -> i32;
    pub fn FSOpenDir(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        path: *const c_char,
        handle: *mut i32,
        err_mask: u32,
    ) -> i32;
    pub fn FSReadDir(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        entry: *mut FSDirEntry,
        err_mask: u32,
    ) -> i32;
    pub fn FSCloseDir(client: *mut FSClient, block: *mut FSCmdBlock, handle: i32, err_mask: u32) -> i32;
    pub fn FSMakeDir(client: *mut FSClient, block: *mut FSCmdBlock, path: *const c_char, err_mask: u32) -> i32;
    pub fn FSRemove(client: *mut FSClient, block: *mut FSCmdBlock, path: *const c_char, err_mask: u32) -> i32;
    pub fn FSRename(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        old_path: *const c_char,
        new_path: *const c_char,
        err_mask: u32,
    ) -> i32;

    pub fn FSChangeDir(client: *mut FSClient, block: *mut FSCmdBlock, path: *const c_char, err_mask: u32) -> i32;
    pub fn FSGetCwd(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        path: *mut c_char,
        len: u32,
        err_mask: u32,
    ) -> i32;
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

#[repr(C, align(8))]
pub struct OSThread([u8; 0x6A0]);

pub type OSThreadProc = extern "C" fn(int_arg: i32, ptr_arg: *mut c_void) -> i32;
pub type OSThreadDeallocator = extern "C" fn(thread: *mut OSThread, stack_end: *mut c_void);

#[repr(C)]
pub struct OSFastMutex([u32; 11]);
impl OSFastMutex {
    pub const ZEROED: Self = Self([0x664D7458, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[repr(C)]
pub struct OSEvent([u32; 9]);
impl OSEvent {
    pub const ZEROED: Self = Self([0x65566E54, 0, 0, 0, 0, 0, 0, 0, 0]);
}

pub const OS_EVENT_MANUAL: i32 = 0;

#[repr(C)]
pub struct FSClient([u32; 0x5C0]);
impl FSClient {
    pub const ZEROED: Self = Self([0; 0x5C0]);
}

#[repr(C)]
pub struct FSCmdBlock([u32; 0x2A0]);

#[repr(C)]
pub struct FSMountSource([u32; 161]);

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct FSStat {
    pub flag: u32,
    pub permission: u32,
    pub owner_id: u32,
    pub group_id: u32,
    pub size: u32,
    pub alloc_size: u32,
    pub quota_size: u64,
    pub ent_id: u32,
    /// Microseconds since 1980-01-01.
    pub ctime: u64,
    pub mtime: u64,
    pub attributes: [u8; 48],
}

#[repr(C, packed)]
pub struct FSDirEntry {
    pub stat: FSStat,
    pub name: [u8; 256],
}

pub const FS_RET_ALL_ERROR: u32 = u32::MAX;
pub const FS_SOURCETYPE_EXTERNAL: u32 = 0;
pub const FS_MAX_MOUNTPATH_SIZE: usize = 128;
pub const FS_MAX_FULLPATH_SIZE: usize = 639;
pub const FS_MAX_ARGPATH_SIZE: usize = FS_MAX_FULLPATH_SIZE;

pub const FS_STAT_DIRECTORY: u32 = 0x8000_0000;
pub const FS_STAT_CTIME: u32 = 0x0800_0000;
pub const FS_STAT_MTIME: u32 = 0x0400_0000;

pub const FS_STATUS_END: i32 = -2;
pub const FS_STATUS_MAX: i32 = -3;
pub const FS_STATUS_ALREADY_OPEN: i32 = -4;
pub const FS_STATUS_EXISTS: i32 = -5;
pub const FS_STATUS_NOT_FOUND: i32 = -6;
pub const FS_STATUS_NOT_FILE: i32 = -7;
pub const FS_STATUS_NOT_DIR: i32 = -8;
pub const FS_STATUS_ACCESS_ERROR: i32 = -9;
pub const FS_STATUS_PERMISSION_ERROR: i32 = -10;
pub const FS_STATUS_FILE_TOO_BIG: i32 = -11;
pub const FS_STATUS_STORAGE_FULL: i32 = -12;
pub const FS_STATUS_UNSUPPORTED_CMD: i32 = -14;
pub const FS_STATUS_WRITE_PROTECTED: i32 = -19;
