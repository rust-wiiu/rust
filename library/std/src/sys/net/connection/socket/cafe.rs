use self::netc::{MSG_PEEK, sockaddr, socklen_t};
use super::{getsockopt, setsockopt, socket_addr_from_c, socket_addr_to_c};
use crate::ffi::{c_int, c_long};
use crate::io::{self, BorrowedBuf, BorrowedCursor, IoSlice, IoSliceMut};
use crate::net::{Shutdown, SocketAddr};
use crate::sync::Once;
use crate::sys::pal::unsupported;
use crate::time::Duration;
use crate::{cmp, mem, ptr};

pub(super) mod netc {
    use crate::ffi::{c_int, c_uint};
    pub use crate::sys::pal::abi::sockets::*;

    pub const EMSGSIZE: c_int = SO_EMSGSIZE;

    // nsysnet only supports IPv4. The shared socket code still needs the IPv6
    // items, so they use the Linux values, and `Socket::new` rejects AF_INET6.
    pub const AF_INET6: c_int = 10;
    pub const IPPROTO_IPV6: c_int = 41;
    pub const IPV6_V6ONLY: c_int = 26;
    pub const IPV6_MULTICAST_LOOP: c_int = 19;
    pub const IPV6_ADD_MEMBERSHIP: c_int = 20;
    pub const IPV6_DROP_MEMBERSHIP: c_int = 21;

    // nsysnet has no timeout options; the shared code only passes these back to
    // `set_timeout`/`timeout`, which report them as unsupported.
    pub const SO_SNDTIMEO: c_int = 0x1005;
    pub const SO_RCVTIMEO: c_int = 0x1006;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct in6_addr {
        pub s6_addr: [u8; 16],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct sockaddr_in6 {
        pub sin6_family: sa_family_t,
        pub sin6_port: u16,
        pub sin6_flowinfo: u32,
        pub sin6_addr: in6_addr,
        pub sin6_scope_id: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct ipv6_mreq {
        pub ipv6mr_multiaddr: in6_addr,
        pub ipv6mr_interface: c_uint,
    }

    /// nsysnet only returns IPv4 addresses and rejects any other length in
    /// `accept`, so this has exactly the size of `sockaddr_in`.
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct sockaddr_storage {
        pub ss_family: sa_family_t,
        pub ss_data: [u8; 14],
    }
}

#[expect(non_camel_case_types)]
pub type wrlen_t = c_int;

pub fn cvt(ret: c_int) -> io::Result<c_int> {
    if ret < 0 { Err(last_error()) } else { Ok(ret) }
}

pub fn cvt_r(f: impl FnOnce() -> c_int) -> io::Result<c_int> {
    cvt(f())
}

pub fn cvt_gai(err: c_int) -> io::Result<()> {
    use io::ErrorKind::Uncategorized;
    Err(match err {
        0 => return Ok(()),
        netc::EAI_AGAIN => io::const_error!(Uncategorized, "temporary failure in name resolution"),
        netc::EAI_FAIL => io::const_error!(Uncategorized, "non-recoverable failure in name resolution"),
        netc::EAI_MEMORY => io::const_error!(Uncategorized, "memory allocation failure"),
        netc::EAI_NONAME => io::const_error!(Uncategorized, "name or service not known"),
        _ => io::const_error!(Uncategorized, "failed to look up address information"),
    })
}

fn last_error() -> io::Error {
    io::Error::from_raw_os_error(unsafe { netc::socketlasterr() })
}

fn to_timeval(dur: Duration) -> netc::timeval {
    // SOSelect rejects components above 999999.
    let mut tv = netc::timeval {
        tv_sec: dur.as_secs().min(999_999) as c_long,
        tv_usec: dur.subsec_micros() as c_long,
    };
    if tv.tv_sec == 0 && tv.tv_usec == 0 {
        tv.tv_usec = 1;
    }
    tv
}

pub fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        netc::socket_lib_init();
        // Brings up the console's configured connection. If that fails, the
        // socket calls report the error.
        netc::ACInitialize();
        netc::ACConnect();
    });
}

pub struct Socket(c_int);

impl Socket {
    pub fn new(family: c_int, ty: c_int) -> io::Result<Socket> {
        if family != netc::AF_INET {
            return Err(io::const_error!(io::ErrorKind::Unsupported, "only IPv4 is supported"));
        }
        cvt(unsafe { netc::socket(family, ty, 0) }).map(Socket)
    }

    pub fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        let (addr, len) = socket_addr_to_c(addr);
        cvt(unsafe { netc::connect(self.0, addr.as_ptr(), len) }).map(drop)
    }

    pub fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> io::Result<()> {
        if timeout.is_zero() {
            return Err(io::Error::ZERO_TIMEOUT);
        }
        if !(0..u32::BITS as c_int).contains(&self.0) {
            return Err(io::const_error!(io::ErrorKind::Unsupported, "socket cannot be polled"));
        }

        self.set_nonblocking(true)?;
        let r = self.connect(addr);
        self.set_nonblocking(false)?;
        match r {
            Ok(()) => return Ok(()),
            Err(e) if e.raw_os_error() == Some(netc::SO_EINPROGRESS) => {}
            Err(e) => return Err(e),
        }

        let mut writefds = netc::fd_set { fd_bits: 1 << self.0 };
        let mut timeout = to_timeval(timeout);
        let n = cvt(unsafe {
            netc::select(self.0 + 1, ptr::null_mut(), &mut writefds, ptr::null_mut(), &mut timeout)
        })?;
        if n == 0 {
            return Err(io::const_error!(io::ErrorKind::TimedOut, "connection timed out"));
        }
        match self.take_error()? {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    pub fn accept(&self, storage: *mut sockaddr, len: *mut socklen_t) -> io::Result<Socket> {
        cvt(unsafe { netc::accept(self.0, storage, len) }).map(Socket)
    }

    pub fn duplicate(&self) -> io::Result<Socket> {
        unsupported()
    }

    fn recv_with_flags(&self, mut buf: BorrowedCursor<'_, u8>, flags: c_int) -> io::Result<()> {
        let len = cmp::min(buf.capacity(), c_int::MAX as usize) as c_int;
        let ret = cvt(unsafe { netc::recv(self.0, buf.as_mut().as_mut_ptr().cast(), len, flags) })?;
        unsafe { buf.advance(ret as usize) };
        Ok(())
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut buf = BorrowedBuf::from(buf);
        self.recv_with_flags(buf.unfilled(), 0)?;
        Ok(buf.len())
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut buf = BorrowedBuf::from(buf);
        self.recv_with_flags(buf.unfilled(), MSG_PEEK)?;
        Ok(buf.len())
    }

    pub fn read_buf(&self, buf: BorrowedCursor<'_, u8>) -> io::Result<()> {
        self.recv_with_flags(buf, 0)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        crate::io::default_read_vectored(|b| self.read(b), bufs)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        false
    }

    fn recv_from_with_flags(
        &self,
        buf: &mut [u8],
        flags: c_int,
    ) -> io::Result<(usize, SocketAddr)> {
        let mut storage: netc::sockaddr_storage = unsafe { mem::zeroed() };
        let mut addrlen = size_of_val(&storage) as socklen_t;
        let len = cmp::min(buf.len(), c_int::MAX as usize) as c_int;
        let n = cvt(unsafe {
            netc::recvfrom(
                self.0,
                buf.as_mut_ptr().cast(),
                len,
                flags,
                (&raw mut storage).cast(),
                &mut addrlen,
            )
        })?;
        Ok((n as usize, unsafe { socket_addr_from_c(&storage, addrlen as usize)? }))
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, 0)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, MSG_PEEK)
    }

    fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let len = cmp::min(buf.len(), c_int::MAX as usize) as c_int;
        cvt(unsafe { netc::send(self.0, buf.as_ptr().cast(), len, 0) }).map(|n| n as usize)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        crate::io::default_write_vectored(|b| self.send(b), bufs)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn set_timeout(&self, _: Option<Duration>, _: c_int) -> io::Result<()> {
        unsupported()
    }

    pub fn timeout(&self, _: c_int) -> io::Result<Option<Duration>> {
        unsupported()
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let how = match how {
            Shutdown::Write => netc::SHUT_WR,
            Shutdown::Read => netc::SHUT_RD,
            Shutdown::Both => netc::SHUT_RDWR,
        };
        cvt(unsafe { netc::shutdown(self.0, how) }).map(drop)
    }

    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        let linger = netc::linger {
            l_onoff: linger.is_some() as c_int,
            l_linger: linger.unwrap_or_default().as_secs().try_into().unwrap_or(c_int::MAX),
        };
        unsafe { setsockopt(self, netc::SOL_SOCKET, netc::SO_LINGER, linger) }
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        let val: netc::linger = unsafe { getsockopt(self, netc::SOL_SOCKET, netc::SO_LINGER)? };
        Ok((val.l_onoff != 0).then(|| Duration::from_secs(val.l_linger as u64)))
    }

    pub fn set_keepalive(&self, keepalive: bool) -> io::Result<()> {
        unsafe { setsockopt(self, netc::SOL_SOCKET, netc::SO_KEEPALIVE, keepalive as c_int) }
    }

    pub fn keepalive(&self) -> io::Result<bool> {
        let raw: c_int = unsafe { getsockopt(self, netc::SOL_SOCKET, netc::SO_KEEPALIVE)? };
        Ok(raw != 0)
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        unsafe { setsockopt(self, netc::IPPROTO_TCP, netc::TCP_NODELAY, nodelay as c_int) }
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        let raw: c_int = unsafe { getsockopt(self, netc::IPPROTO_TCP, netc::TCP_NODELAY)? };
        Ok(raw != 0)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        unsafe { setsockopt(self, netc::SOL_SOCKET, netc::SO_NONBLOCK, nonblocking as c_int) }
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        let raw: c_int = unsafe { getsockopt(self, netc::SOL_SOCKET, netc::SO_ERROR)? };
        Ok((raw != 0).then(|| io::Error::from_raw_os_error(raw)))
    }

    pub fn as_raw(&self) -> c_int {
        self.0
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe { netc::socketclose(self.0) };
    }
}
