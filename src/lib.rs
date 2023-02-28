use libc::{c_void, size_t, ssize_t};
use std::{ffi::c_int, isize, mem::size_of, ptr::addr_of_mut};

pub const IPPROTO_HOMA: i32 = 0xFD;
pub const SO_HOMA_SET_BUF: i32 = 10;
pub const HOMA_BPAGE_SHIFT: usize = 16;
pub const HOMA_BPAGE_SIZE: usize = 1 << HOMA_BPAGE_SHIFT;
pub const HOMA_MAX_MESSAGE_LENGTH: usize = 1000000;
pub const HOMA_MAX_BPAGES: usize =
    (HOMA_MAX_MESSAGE_LENGTH + HOMA_BPAGE_SIZE - 1) >> HOMA_BPAGE_SHIFT;

pub const HOMA_RECVMSG_REQUEST: c_int = 0x01;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct homa_set_buf_args {
    pub start: *mut c_void,
    pub length: size_t,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct homa_sendmsg_args {
    id: u64,
    completion_cookie: u64,
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

pub fn homa_socket(domain: c_int) -> i32 {
    unsafe { libc::socket(domain, libc::SOCK_DGRAM, IPPROTO_HOMA) }
}

pub fn homa_send(
    sockfd: c_int,
    message_buf: *mut c_void,
    length: size_t,
    dest_addr: *const libc::sockaddr_storage,
    id: *mut u64,
    completion_cookie: u64,
) -> ssize_t {
    let mut args = homa_sendmsg_args {
        id: 0,
        completion_cookie,
    };
    let mut vec = libc::iovec {
        iov_base: message_buf,
        iov_len: length,
    };
    let mut hdr = libc::msghdr {
        msg_name: dest_addr as *mut c_void,
        msg_namelen: size_of::<libc::sockaddr_storage>() as u32,
        msg_iov: addr_of_mut!(vec),
        msg_iovlen: 1,
        msg_control: addr_of_mut!(args) as *mut c_void,
        msg_controllen: 0,
        msg_flags: 0,
    };
    let result = unsafe { libc::sendmsg(sockfd, addr_of_mut!(hdr), 0) };
    if result >= 0 && !id.is_null() {
        unsafe { *id = args.id };
    }
    result
}
