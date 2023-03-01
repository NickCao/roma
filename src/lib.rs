use libc::{c_void, size_t};
use memmap2::{MmapMut, MmapOptions};
use socket2::{Domain, SockAddr, Socket, Type};
use std::cmp::max;
use std::io::{Error, IoSlice, Result};
use std::net::SocketAddr;
use std::os::fd::AsRawFd;
use std::{ffi::c_int, mem::size_of};

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
        dest_addr: SocketAddr,
        bufs: &[IoSlice<'_>],
        id: u64,
        completion_cookie: u64,
    ) -> Result<u64> {
        let dest_addr: SockAddr = dest_addr.into();
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

    pub fn recv(
        &self,
        id: u64,
        flags: c_int,
        bufs: &[IoSlice<'_>],
    ) -> Result<(Vec<IoSlice<'_>>, Option<SocketAddr>)> {
        let src_addr: libc::sockaddr_storage = unsafe { std::mem::zeroed() };

        let mut bpage_offsets = [0; HOMA_MAX_BPAGES];
        unsafe {
            let bufs: Vec<u32> = bufs
                .iter()
                .map(|x| x.as_ptr().offset_from(self.buffer.as_ptr()) as u32)
                .collect();
            bpage_offsets[..bufs.len()].copy_from_slice(&bufs);
        }

        let recvmsg_args = homa_recvmsg_args {
            id,
            completion_cookie: 0,
            flags,
            num_bpages: bufs.len() as u32,
            pad: [0; 2],
            bpage_offsets,
        };
        let mut hdr = libc::msghdr {
            msg_name: &src_addr as *const _ as *mut _,
            msg_namelen: size_of::<libc::sockaddr_storage>() as u32,
            msg_iov: std::ptr::null_mut() as *mut _,
            msg_iovlen: 0,
            msg_control: &recvmsg_args as *const _ as *mut c_void,
            msg_controllen: size_of::<homa_recvmsg_args>(),
            msg_flags: 0,
        };

        let length = unsafe { libc::recvmsg(self.socket.as_raw_fd(), &mut hdr, 0) };
        if length < 0 {
            return Err(Error::last_os_error());
        }
        let mut length: usize = length.try_into().unwrap();

        let mut iovec = vec![];
        for i in 0..recvmsg_args.num_bpages as usize {
            let size = max(length, HOMA_MAX_BPAGES);
            iovec.push(unsafe {
                IoSlice::new(std::slice::from_raw_parts(
                    self.buffer
                        .as_ptr()
                        .offset(recvmsg_args.bpage_offsets[i] as isize),
                    size,
                ))
            });
            length -= size;
        }

        Ok((iovec, unsafe {
            SockAddr::new(
                src_addr,
                size_of::<libc::sockaddr_storage>().try_into().unwrap(),
            )
            .as_socket()
        }))
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct homa_set_buf_args {
    pub start: *mut c_void,
    pub length: size_t,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct homa_sendmsg_args {
    id: u64,
    completion_cookie: u64,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct homa_recvmsg_args {
    pub id: u64,
    pub completion_cookie: u64,
    pub flags: c_int,
    pub num_bpages: u32,
    pub pad: [u32; 2],
    pub bpage_offsets: [u32; HOMA_MAX_BPAGES],
}
