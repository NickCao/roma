#![feature(int_roundings)]
#![feature(default_free_fn)]

use libc::c_void;
use memmap2::{MmapMut, MmapOptions};
use nix::sys::socket::setsockopt;
use socket2::{Domain, SockAddr, Socket, Type};
use std::cmp::min;
use std::collections::VecDeque;
use std::io::{Error, IoSlice, Result};
use std::net::SocketAddr;
use std::os::fd::AsRawFd;
use std::slice;
use std::{ffi::c_int, mem::size_of};


use crate::types::HomaBuf;

pub mod consts;
pub mod types;

pub struct HomaSocket {
    pub socket: Socket,
    buf: MmapMut,
    backlog: VecDeque<u32>,
}

impl HomaSocket {
    pub fn new(domain: Domain, pages: usize) -> Result<Self> {
        log::debug!("HomaSocket::new(domain: {:?}, pages: {})", domain, pages);
        let socket = Socket::new_raw(domain, Type::DGRAM, Some(consts::IPPROTO_HOMA.into()))?;

        let length = pages * consts::HOMA_BPAGE_SIZE;
        let buffer = MmapOptions::new().len(length).map_anon()?;

        setsockopt(socket.as_raw_fd(), HomaBuf, &buffer).unwrap();

        Ok(Self {
            socket,
            buf: buffer,
            backlog: VecDeque::default(),
        })
    }

    pub fn send(
        &self,
        dest_addr: SocketAddr,
        bufs: &[u8],
        id: u64,
        completion_cookie: u64,
    ) -> Result<u64> {
        log::debug!(
            "HomaSocket::send(dest_addr: {}, bufs: {}, id: {}, completion_cookie: {})",
            dest_addr,
            bufs.len(),
            id,
            completion_cookie
        );
        let dest_addr: SockAddr = dest_addr.into();
        let sendmsg_args = types::homa_sendmsg_args {
            id,
            completion_cookie,
        };
        let hdr = libc::msghdr {
            msg_name: dest_addr.as_ptr() as *mut _,
            msg_namelen: dest_addr.len(),
            msg_iov: [IoSlice::new(bufs)].as_mut_ptr().cast(),
            msg_iovlen: 1,
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
        &mut self,
        id: u64,
        flags: consts::HomaRecvmsgFlags,
        bufs: &mut [u8],
    ) -> Result<(u64, u64, Option<SocketAddr>)> {
        log::debug!(
            "HomaSocket::recv(id: {}, flags: {:?}, bufs: {})",
            id,
            flags,
            bufs.len(),
        );
        let src_addr: libc::sockaddr_storage = unsafe { std::mem::zeroed() };

        let ret = min(self.backlog.len(), consts::HOMA_MAX_BPAGES);
        let bpages = self.backlog.drain(0..ret);

        let mut bpage_offsets = [0; consts::HOMA_MAX_BPAGES];
        for (i, bpage) in bpages.enumerate() {
            bpage_offsets[i] = bpage;
        }

        let recvmsg_args = types::homa_recvmsg_args {
            id,
            completion_cookie: 0,
            flags: flags.bits(),
            num_bpages: ret.try_into().unwrap(),
            pad: [0; 2],
            bpage_offsets,
        };

        let mut hdr = libc::msghdr {
            msg_name: &src_addr as *const _ as *mut _,
            msg_namelen: size_of::<libc::sockaddr_storage>() as u32,
            msg_iov: std::ptr::null_mut() as *mut _,
            msg_iovlen: 0,
            msg_control: &recvmsg_args as *const _ as *mut c_void,
            msg_controllen: size_of::<types::homa_recvmsg_args>(),
            msg_flags: 0,
        };

        let length = unsafe {
            libc::recvmsg(
                self.socket.as_raw_fd(),
                &mut hdr,
                0, // flags are ignored
            )
        };
        if length < 0 {
            return Err(Error::last_os_error());
        }
        let length: usize = length.try_into().unwrap();

        for i in 0..recvmsg_args.num_bpages as usize {
            let len = if i != recvmsg_args.num_bpages as usize - 1 {
                consts::HOMA_BPAGE_SIZE
            } else {
                length - consts::HOMA_BPAGE_SIZE * (recvmsg_args.num_bpages as usize - 1)
            };
            let offset = recvmsg_args.bpage_offsets[i];
            unsafe {
                self.backlog.push_back(offset);
                let data = self.buf.as_ptr().offset(offset.try_into().unwrap());
                bufs[i * consts::HOMA_BPAGE_SIZE..i * consts::HOMA_BPAGE_SIZE + len]
                    .copy_from_slice(slice::from_raw_parts(data, len));
            }
        }

        Ok((recvmsg_args.id, recvmsg_args.completion_cookie, unsafe {
            SockAddr::new(
                src_addr,
                size_of::<libc::sockaddr_storage>().try_into().unwrap(),
            )
            .as_socket()
        }))
    }

    pub fn abort(&self, id: u64, error: c_int) -> nix::Result<i32> {
        let mut abort_args = types::homa_abort_args::new(id, error);
        unsafe { types::homa_abort(self.socket.as_raw_fd(), &mut abort_args) }
    }

    pub fn freeze(&self) -> nix::Result<i32> {
        unsafe { types::homa_freeze(self.socket.as_raw_fd()) }
    }
}
