use std::ffi::c_int;

/// Homa's protocol number within the IP protocol space (this is not an officially allocated slot).
pub const IPPROTO_HOMA: c_int = 0xFD;

/// Maximum bytes of payload in a Homa request or response message.
pub const HOMA_MAX_MESSAGE_LENGTH: usize = 1000000;

/// Number of bytes in pages used for receive buffers. Must be power of two.
pub const HOMA_BPAGE_SIZE: usize = 1 << 16;

/// The largest number of bpages that will be required to store an incoming message.
pub const HOMA_MAX_BPAGES: usize = HOMA_MAX_MESSAGE_LENGTH.div_ceil(HOMA_BPAGE_SIZE);

pub const HOMA_RECVMSG_REQUEST: c_int = 0x01;
pub const HOMA_RECVMSG_RESPONSE: c_int = 0x02;
pub const HOMA_RECVMSG_NONBLOCKING: c_int = 0x04;
pub const HOMA_RECVMSG_VALID_FLAGS: c_int = 0x07;

/// setsockopt option for specifying buffer region.
pub const SO_HOMA_SET_BUF: i32 = 10;
