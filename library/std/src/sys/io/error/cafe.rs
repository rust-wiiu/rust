//! Raw OS errors on Cafe are nsysnet socket error codes.

use cafe_sys::nsys::net::socket::Error;

use crate::{fmt, io};

pub fn errno() -> i32 {
    0
}

pub fn is_interrupted(_code: i32) -> bool {
    false
}

pub fn decode_error_kind(code: i32) -> io::ErrorKind {
    use io::ErrorKind::*;
    match Error(code) {
        Error::ETIMEDOUT => TimedOut,
        Error::EOPNOTSUPP
        | Error::EAFNOSUPPORT
        | Error::EPROTONOSUPPORT
        | Error::ENOPROTOOPT => Unsupported,
        Error::ECONNABORTED | Error::EABORTED => ConnectionAborted,
        Error::EWOULDBLOCK => WouldBlock,
        Error::ECONNREFUSED => ConnectionRefused,
        Error::ECONNRESET => ConnectionReset,
        Error::ENOTCONN => NotConnected,
        Error::EINVAL => InvalidInput,
        Error::EPIPE | Error::ESHUTDOWN => BrokenPipe,
        Error::ENOMEM => OutOfMemory,
        Error::EADDRNOTAVAIL => AddrNotAvailable,
        Error::EADDRINUSE => AddrInUse,
        Error::EINPROGRESS => InProgress,
        Error::ENETUNREACH => NetworkUnreachable,
        Error::EBUSY => ResourceBusy,
        _ => Uncategorized,
    }
}

pub fn format_error(code: i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match Error(code) {
        Error::ENOBUFS => "no buffer space available",
        Error::ETIMEDOUT => "connection timed out",
        Error::EOPNOTSUPP => "operation not supported",
        Error::ECONNABORTED => "connection aborted",
        Error::EWOULDBLOCK => "operation would block",
        Error::ECONNREFUSED => "connection refused",
        Error::ECONNRESET => "connection reset by peer",
        Error::ENOTCONN => "socket is not connected",
        Error::EINVAL => "invalid argument",
        Error::EMSGSIZE => "message too long",
        Error::EPIPE => "broken pipe",
        Error::ESHUTDOWN => "socket is shut down",
        Error::ENOMEM => "out of memory",
        Error::EADDRNOTAVAIL => "address not available",
        Error::EADDRINUSE => "address in use",
        Error::EAFNOSUPPORT => "address family not supported",
        Error::EINPROGRESS => "operation in progress",
        Error::ENETUNREACH => "network unreachable",
        Error::EPROTONOSUPPORT => "protocol not supported",
        Error::EBUSY => "resource busy",
        Error::EISCONN => "socket is already connected",
        Error::EALREADY => "operation already in progress",
        Error::ENOPROTOOPT => "protocol option not available",
        Error::ENOTSOCK => "not a socket",
        Error::ENORESOURCES => "out of select resources",
        Error::EABORTED => "operation aborted",
        Error::EMFILE => "too many open sockets",
        _ => return write!(f, "socket error {code}"),
    })
}
