use libc::{c_void, size_t};
use memmap2::{Mmap, MmapMut, MmapOptions};
use socket2::{Domain, SockAddr, Socket, Type};
use std::io::{Error, ErrorKind, IoSlice, Result};
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
    pub buffer: MmapMut,
}

impl HomaSocket {
    pub fn new(domain: Domain, pages: usize) -> Result<Self> {
        let socket = Socket::new_raw(domain, Type::DGRAM, Some(IPPROTO_HOMA.into()))?;

        let length = pages * HOMA_BPAGE_SIZE;
        let mut buffer = MmapOptions::new().len(length).map_anon()?;

        let set_buf_args = homa_set_buf_args {
            start: buffer.as_mut_ptr() as *mut c_void,
            length: buffer.len(),
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

        Ok(Self { socket, buffer })
    }

    pub fn send(
        &self,
        dest_addr: &SockAddr,
        bufs: &[IoSlice<'_>],
        id: u64,
        completion_cookie: u64,
    ) -> Result<u64> {
        let sendmsg_args = homa_sendmsg_args {
            id,
            completion_cookie,
        };
        let hdr = libc::msghdr {
            msg_name: dest_addr.as_ptr() as *mut _,
            msg_namelen: dest_addr.len(),
            msg_iov: bufs.as_ptr() as *mut _,
            msg_iovlen: bufs.len(),
            msg_control: &sendmsg_args as *const _ as *mut _,
            msg_controllen: 0,
            msg_flags: 0,
        };
        let result = unsafe { libc::sendmsg(self.socket.as_raw_fd(), &hdr, 0) };
        if result < 0 {
            return Err(Error::last_os_error());
        }
        Ok(sendmsg_args.id)
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
