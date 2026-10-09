//! Raw OS errors on Cafe are nsysnet socket error codes.

use crate::sys::pal::abi::sockets as netc;
use crate::{fmt, io};

pub fn errno() -> i32 {
    0
}

pub fn is_interrupted(_code: i32) -> bool {
    false
}

pub fn decode_error_kind(code: i32) -> io::ErrorKind {
    use io::ErrorKind::*;
    match code {
        netc::SO_ETIMEDOUT => TimedOut,
        netc::SO_EOPNOTSUPP | netc::SO_EAFNOSUPPORT | netc::SO_EPROTONOSUPPORT | netc::SO_ENOPROTOOPT => Unsupported,
        netc::SO_ECONNABORTED | netc::SO_EABORTED => ConnectionAborted,
        netc::SO_EWOULDBLOCK => WouldBlock,
        netc::SO_ECONNREFUSED => ConnectionRefused,
        netc::SO_ECONNRESET => ConnectionReset,
        netc::SO_ENOTCONN => NotConnected,
        netc::SO_EINVAL => InvalidInput,
        netc::SO_EPIPE | netc::SO_ESHUTDOWN => BrokenPipe,
        netc::SO_ENOMEM => OutOfMemory,
        netc::SO_EADDRNOTAVAIL => AddrNotAvailable,
        netc::SO_EADDRINUSE => AddrInUse,
        netc::SO_EINPROGRESS => InProgress,
        netc::SO_ENETUNREACH => NetworkUnreachable,
        netc::SO_EBUSY => ResourceBusy,
        _ => Uncategorized,
    }
}

pub fn format_error(code: i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match code {
        netc::SO_ENOBUFS => "no buffer space available",
        netc::SO_ETIMEDOUT => "connection timed out",
        netc::SO_EOPNOTSUPP => "operation not supported",
        netc::SO_ECONNABORTED => "connection aborted",
        netc::SO_EWOULDBLOCK => "operation would block",
        netc::SO_ECONNREFUSED => "connection refused",
        netc::SO_ECONNRESET => "connection reset by peer",
        netc::SO_ENOTCONN => "socket is not connected",
        netc::SO_EINVAL => "invalid argument",
        netc::SO_EMSGSIZE => "message too long",
        netc::SO_EPIPE => "broken pipe",
        netc::SO_ESHUTDOWN => "socket is shut down",
        netc::SO_ENOMEM => "out of memory",
        netc::SO_EADDRNOTAVAIL => "address not available",
        netc::SO_EADDRINUSE => "address in use",
        netc::SO_EAFNOSUPPORT => "address family not supported",
        netc::SO_EINPROGRESS => "operation in progress",
        netc::SO_ENETUNREACH => "network unreachable",
        netc::SO_EPROTONOSUPPORT => "protocol not supported",
        netc::SO_EBUSY => "resource busy",
        netc::SO_EISCONN => "socket is already connected",
        netc::SO_EALREADY => "operation already in progress",
        netc::SO_ENOPROTOOPT => "protocol option not available",
        netc::SO_ENOTSOCK => "not a socket",
        netc::SO_ENORESOURCES => "out of select resources",
        netc::SO_EABORTED => "operation aborted",
        netc::SO_EMFILE => "too many open sockets",
        _ => return write!(f, "socket error {code}"),
    })
}
