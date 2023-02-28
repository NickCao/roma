use libc::{c_void, size_t};
use socket2::{Domain, Socket, Type};
use std::io::{Error, ErrorKind, Result};
use std::os::fd::AsRawFd;
use std::{ffi::c_int, isize, mem::size_of, ptr::addr_of_mut};

pub const IPPROTO_HOMA: i32 = 0xFD;
pub const SO_HOMA_SET_BUF: i32 = 10;
pub const HOMA_BPAGE_SHIFT: usize = 16;
pub const HOMA_BPAGE_SIZE: usize = 1 << HOMA_BPAGE_SHIFT;
pub const HOMA_MAX_MESSAGE_LENGTH: usize = 1000000;
pub const HOMA_MAX_BPAGES: usize =
    (HOMA_MAX_MESSAGE_LENGTH + HOMA_BPAGE_SIZE - 1) >> HOMA_BPAGE_SHIFT;

pub const HOMA_RECVMSG_REQUEST: c_int = 0x01;

pub struct HomaSocket {
    pub socket: Socket,
}

impl HomaSocket {
    pub fn new(domain: Domain, pages: usize) -> Result<Self> {
        let socket = Socket::new_raw(domain, Type::DGRAM, Some(IPPROTO_HOMA.into()))?;

        let length = pages * HOMA_BPAGE_SIZE;
        let buffer = unsafe {
            libc::mmap(
                std::ptr::null_mut() as *mut c_void,
                length,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                0,
                0,
            )
        };

        assert_ne!(buffer, libc::MAP_FAILED);

        let set_buf_args = homa_set_buf_args {
            start: buffer,
            length,
        };

        let result = unsafe {
            libc::setsockopt(
                socket.as_raw_fd(),
                IPPROTO_HOMA,
                SO_HOMA_SET_BUF,
                &set_buf_args as *const homa_set_buf_args as *const c_void,
                size_of::<homa_set_buf_args>() as u32,
            )
        };

        assert!(result >= 0);

        Ok(Self { socket })
    }
}

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
) -> isize {
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

pub fn homa_reply(
    sockfd: c_int,
    message_buf: *mut c_void,
    length: size_t,
    dest_addr: *mut libc::sockaddr_storage,
    id: u64,
) -> isize {
    let mut args = homa_sendmsg_args {
        id,
        completion_cookie: 0,
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

    unsafe { libc::sendmsg(sockfd, addr_of_mut!(hdr), 0) }
}
