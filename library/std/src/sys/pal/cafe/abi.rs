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

imports_section!("nn_temp", ["TEMPInit", "TEMPCreateAndInitTempDir", "TEMPGetDirGlobalPath"], []);

imports_section!(
    "nsysnet",
    [
        "socket_lib_init",
        "socketlasterr",
        "socket",
        "socketclose",
        "connect",
        "bind",
        "listen",
        "accept",
        "recv",
        "recvfrom",
        "send",
        "sendto",
        "shutdown",
        "getsockopt",
        "setsockopt",
        "getpeername",
        "getsockname",
        "select",
        "getaddrinfo",
        "freeaddrinfo"
    ],
    []
);

imports_section!("nn_ac", ["ACInitialize", "ACConnect"], []);

imports_section!("nlibnss", ["NSSInit", "NSSGetRandom"], []);

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
    pub fn FSCloseFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        err_mask: u32,
    ) -> i32;
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
    pub fn FSTruncateFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        err_mask: u32,
    ) -> i32;
    pub fn FSFlushFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        err_mask: u32,
    ) -> i32;
    pub fn FSGetPosFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        pos: *mut u32,
        err_mask: u32,
    ) -> i32;
    pub fn FSSetPosFile(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        pos: u32,
        err_mask: u32,
    ) -> i32;
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
    pub fn FSCloseDir(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        handle: i32,
        err_mask: u32,
    ) -> i32;
    pub fn FSMakeDir(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        path: *const c_char,
        err_mask: u32,
    ) -> i32;
    pub fn FSRemove(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        path: *const c_char,
        err_mask: u32,
    ) -> i32;
    pub fn FSRename(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        old_path: *const c_char,
        new_path: *const c_char,
        err_mask: u32,
    ) -> i32;

    pub fn FSChangeDir(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        path: *const c_char,
        err_mask: u32,
    ) -> i32;
    pub fn FSGetCwd(
        client: *mut FSClient,
        block: *mut FSCmdBlock,
        path: *mut c_char,
        len: u32,
        err_mask: u32,
    ) -> i32;

    pub fn TEMPInit() -> i32;
    pub fn TEMPCreateAndInitTempDir(max_size: u32, pref: u32, dir_id: *mut u64) -> i32;
    pub fn TEMPGetDirGlobalPath(dir_id: u64, path: *mut c_char, len: u32) -> i32;

    pub fn NSSInit() -> i32;
    pub fn NSSGetRandom(buf: *mut u8, len: i32) -> i32;
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

pub const TEMP_DEVICE_PREFERENCE_FREE_SPACE: u32 = 0;

#[expect(non_camel_case_types)]
pub mod sockets {
    use crate::ffi::{c_char, c_int, c_long, c_void};

    pub type sa_family_t = u16;
    pub type socklen_t = c_int;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct sockaddr {
        pub sa_family: sa_family_t,
        pub sa_data: [c_char; 14],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct in_addr {
        pub s_addr: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct sockaddr_in {
        pub sin_family: sa_family_t,
        pub sin_port: u16,
        pub sin_addr: in_addr,
        pub sin_zero: [u8; 8],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct ip_mreq {
        pub imr_multiaddr: in_addr,
        pub imr_interface: in_addr,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct linger {
        pub l_onoff: c_int,
        pub l_linger: c_int,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct timeval {
        pub tv_sec: c_long,
        pub tv_usec: c_long,
    }

    /// One bit per descriptor, so `select` only handles descriptors below 32.
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct fd_set {
        pub fd_bits: u32,
    }

    #[repr(C)]
    pub struct addrinfo {
        pub ai_flags: c_int,
        pub ai_family: c_int,
        pub ai_socktype: c_int,
        pub ai_protocol: c_int,
        pub ai_addrlen: socklen_t,
        pub ai_canonname: *mut c_char,
        pub ai_addr: *mut sockaddr,
        pub ai_next: *mut addrinfo,
    }

    pub const AF_INET: c_int = 2;
    pub const SOCK_STREAM: c_int = 1;
    pub const SOCK_DGRAM: c_int = 2;
    pub const IPPROTO_IP: c_int = 0;
    pub const IPPROTO_TCP: c_int = 6;
    pub const MSG_PEEK: c_int = 0x2;
    pub const SHUT_RD: c_int = 0;
    pub const SHUT_WR: c_int = 1;
    pub const SHUT_RDWR: c_int = 2;

    pub const SOL_SOCKET: c_int = -1;
    pub const SO_REUSEADDR: c_int = 0x0004;
    pub const SO_KEEPALIVE: c_int = 0x0008;
    pub const SO_BROADCAST: c_int = 0x0020;
    pub const SO_LINGER: c_int = 0x0080;
    pub const SO_ERROR: c_int = 0x1007;
    pub const SO_NONBLOCK: c_int = 0x1016;

    pub const IP_TTL: c_int = 4;
    pub const IP_MULTICAST_TTL: c_int = 10;
    pub const IP_MULTICAST_LOOP: c_int = 11;
    pub const IP_ADD_MEMBERSHIP: c_int = 12;
    pub const IP_DROP_MEMBERSHIP: c_int = 13;
    pub const TCP_NODELAY: c_int = 0x2004;

    pub const EAI_AGAIN: c_int = 2;
    pub const EAI_FAIL: c_int = 4;
    pub const EAI_MEMORY: c_int = 6;
    pub const EAI_NONAME: c_int = 8;

    pub const SO_ENOBUFS: c_int = 1;
    pub const SO_ETIMEDOUT: c_int = 2;
    pub const SO_EOPNOTSUPP: c_int = 4;
    pub const SO_ECONNABORTED: c_int = 5;
    pub const SO_EWOULDBLOCK: c_int = 6;
    pub const SO_ECONNREFUSED: c_int = 7;
    pub const SO_ECONNRESET: c_int = 8;
    pub const SO_ENOTCONN: c_int = 9;
    pub const SO_EINVAL: c_int = 11;
    pub const SO_EMSGSIZE: c_int = 12;
    pub const SO_EPIPE: c_int = 13;
    pub const SO_ESHUTDOWN: c_int = 15;
    pub const SO_ENOMEM: c_int = 18;
    pub const SO_EADDRNOTAVAIL: c_int = 19;
    pub const SO_EADDRINUSE: c_int = 20;
    pub const SO_EAFNOSUPPORT: c_int = 21;
    pub const SO_EINPROGRESS: c_int = 22;
    pub const SO_ENETUNREACH: c_int = 30;
    pub const SO_EPROTONOSUPPORT: c_int = 31;
    pub const SO_EBUSY: c_int = 44;
    pub const SO_EISCONN: c_int = 3;
    pub const SO_EALREADY: c_int = 10;
    pub const SO_ENOPROTOOPT: c_int = 16;
    pub const SO_ENOTSOCK: c_int = 24;
    pub const SO_ENORESOURCES: c_int = 48;
    pub const SO_EABORTED: c_int = 50;
    pub const SO_EMFILE: c_int = 51;

    unsafe extern "C" {
        pub fn socket_lib_init() -> c_int;
        pub fn socketlasterr() -> c_int;
        pub fn socket(family: c_int, ty: c_int, protocol: c_int) -> c_int;
        pub fn socketclose(fd: c_int) -> c_int;
        pub fn connect(fd: c_int, addr: *const sockaddr, len: socklen_t) -> c_int;
        pub fn bind(fd: c_int, addr: *const sockaddr, len: socklen_t) -> c_int;
        pub fn listen(fd: c_int, backlog: c_int) -> c_int;
        pub fn accept(fd: c_int, addr: *mut sockaddr, len: *mut socklen_t) -> c_int;
        pub fn recv(fd: c_int, buf: *mut c_void, len: c_int, flags: c_int) -> c_int;
        pub fn recvfrom(
            fd: c_int,
            buf: *mut c_void,
            len: c_int,
            flags: c_int,
            from: *mut sockaddr,
            from_len: *mut socklen_t,
        ) -> c_int;
        pub fn send(fd: c_int, buf: *const c_void, len: c_int, flags: c_int) -> c_int;
        pub fn sendto(
            fd: c_int,
            buf: *const c_void,
            len: c_int,
            flags: c_int,
            dest: *const sockaddr,
            dest_len: socklen_t,
        ) -> c_int;
        pub fn shutdown(fd: c_int, how: c_int) -> c_int;
        pub fn getsockopt(
            fd: c_int,
            level: c_int,
            name: c_int,
            value: *mut c_void,
            len: *mut socklen_t,
        ) -> c_int;
        pub fn setsockopt(
            fd: c_int,
            level: c_int,
            name: c_int,
            value: *const c_void,
            len: socklen_t,
        ) -> c_int;
        pub fn getpeername(fd: c_int, addr: *mut sockaddr, len: *mut socklen_t) -> c_int;
        pub fn getsockname(fd: c_int, addr: *mut sockaddr, len: *mut socklen_t) -> c_int;
        pub fn select(
            nfds: c_int,
            readfds: *mut fd_set,
            writefds: *mut fd_set,
            exceptfds: *mut fd_set,
            timeout: *mut timeval,
        ) -> c_int;
        pub fn getaddrinfo(
            node: *const c_char,
            service: *const c_char,
            hints: *const addrinfo,
            res: *mut *mut addrinfo,
        ) -> c_int;
        pub fn freeaddrinfo(res: *mut addrinfo);

        pub fn ACInitialize() -> i32;
        pub fn ACConnect() -> i32;
    }
}
