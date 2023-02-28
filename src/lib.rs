use libc::{c_void, size_t};
use std::ffi::c_int;

pub const IPPROTO_HOMA: i32 = 0xFD;
pub const SO_HOMA_SET_BUF: i32 = 10;
pub const HOMA_BPAGE_SHIFT: usize = 16;
pub const HOMA_BPAGE_SIZE: usize = 1 << HOMA_BPAGE_SHIFT;
pub const HOMA_MAX_MESSAGE_LENGTH: usize = 1000000;
pub const HOMA_MAX_BPAGES: usize =
    (HOMA_MAX_MESSAGE_LENGTH + HOMA_BPAGE_SIZE - 1) >> HOMA_BPAGE_SHIFT;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct homa_set_buf_args {
    pub start: *mut c_void,
    pub length: size_t,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct homa_recvmsg_args {
    pub id: u64,
    pub completion_cookie: u64,
    pub flags: c_int,
    pub num_bpages: u32,
    pub pad: [u32; 2],
    pub bpage_offsets: [u32; HOMA_MAX_BPAGES],
}
