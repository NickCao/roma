#![feature(int_roundings)]
#![feature(default_free_fn)]

use io_uring::types::Fd;
use io_uring::{cqueue, opcode, squeue, IoUring};
use libc::c_void;
use memmap2::{MmapMut, MmapOptions};
use nix::sys::socket::setsockopt;
use socket2::{Domain, SockAddr, Socket, Type};
use std::cmp::min;
use std::io::{Error, IoSlice, Result};
use std::net::SocketAddr;
use std::os::fd::AsRawFd;
use std::ptr::null_mut;
use std::{ffi::c_int, mem::size_of};

use crate::types::HomaBuf;

pub mod consts;
pub mod types;

pub struct HomaSocket {
    pub socket: Socket,
    buffer: MmapMut,
}

impl HomaSocket {
    pub fn new(domain: Domain, pages: usize) -> Result<Self> {
        log::debug!("HomaSocket::new(domain: {:?}, pages: {})", domain, pages);
        let socket = Socket::new_raw(domain, Type::DGRAM, Some(consts::IPPROTO_HOMA.into()))?;

        let length = pages * consts::HOMA_BPAGE_SIZE;
        let buffer = MmapOptions::new().len(length).map_anon()?;

        setsockopt(socket.as_raw_fd(), HomaBuf, &buffer).unwrap();

        Ok(Self { socket, buffer })
    }

    pub fn roundtrip(
        &self,
        dest_addr: SocketAddr,
        data: &[IoSlice<'_>],
        bufs: &[IoSlice<'_>],
    ) -> Result<Vec<IoSlice<'_>>> {
        log::debug!("HomaSocket::roundtrip(dest_addr: {})", dest_addr);

        let mut bpage_offsets = [0; consts::HOMA_MAX_BPAGES];
        unsafe {
            let bufs: Vec<u32> = bufs
                .iter()
                .map(|x| x.as_ptr().offset_from(self.buffer.as_ptr()) as u32)
                .collect();
            bpage_offsets[..bufs.len()].copy_from_slice(&bufs);
        }

        // abusing the fact that homa_sendmsg_args and homa_recvmsg_args
        // has the same layout in the begining
        let args = types::homa_recvmsg_args {
            id: 0,
            completion_cookie: 0, // omitted for now
            flags: consts::HomaRecvmsgFlags::RESPONSE.bits(),
            num_bpages: bufs.len() as u32,
            pad: [0; 2],
            bpage_offsets,
        };

        let dest_addr: SockAddr = dest_addr.into();
        let send_hdr = libc::msghdr {
            msg_name: dest_addr.as_ptr() as *mut _,
            msg_namelen: dest_addr.len(),
            msg_iov: data.as_ptr() as *mut _,
            msg_iovlen: data.len(),
            msg_control: &args as *const _ as *mut _,
            msg_controllen: 0,
            msg_flags: 0,
        };

        let mut recv_hdr = libc::msghdr {
            msg_name: null_mut(),
            msg_namelen: 0,
            msg_iov: null_mut(),
            msg_iovlen: 0,
            msg_control: &args as *const _ as *mut _,
            msg_controllen: size_of::<types::homa_recvmsg_args>(),
            msg_flags: 0,
        };

        let opsend = opcode::SendMsg::new(Fd(self.socket.as_raw_fd()), &send_hdr)
            .build()
            .user_data(1)
            .flags(squeue::Flags::IO_LINK | squeue::Flags::SKIP_SUCCESS);
        let oprecv = opcode::RecvMsg::new(Fd(self.socket.as_raw_fd()), &mut recv_hdr)
            .build()
            .user_data(2);

        let mut ring = IoUring::new(8)?;
        unsafe {
            let mut queue = ring.submission();
            queue.push(&opsend).unwrap();
            queue.push(&oprecv).unwrap();
        }
        ring.submit_and_wait(1).unwrap();

        dbg!(&args);

        let cq: Vec<cqueue::Entry> = ring.completion().into_iter().collect();
        dbg!(&cq);
        assert_eq!(cq.len(), 1);

        let length = cq.last().unwrap().result();

        let mut length: usize = length.try_into().unwrap();

        let mut iovec = vec![];
        for i in 0..args.num_bpages as usize {
            let size = min(length, consts::HOMA_BPAGE_SIZE);
            iovec.push(unsafe {
                IoSlice::new(std::slice::from_raw_parts(
                    self.buffer.as_ptr().offset(args.bpage_offsets[i] as isize),
                    size,
                ))
            });
            length -= size;
        }

        Ok(iovec)
    }

    pub fn send(
        &self,
        dest_addr: SocketAddr,
        bufs: &[IoSlice<'_>],
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
        flags: consts::HomaRecvmsgFlags,
        bufs: &[IoSlice<'_>],
    ) -> Result<(u64, u64, Vec<IoSlice<'_>>, Option<SocketAddr>)> {
        log::debug!(
            "HomaSocket::recv(id: {}, flags: {:?}, bufs: {})",
            id,
            flags,
            bufs.len(),
        );
        let src_addr: libc::sockaddr_storage = unsafe { std::mem::zeroed() };

        let mut bpage_offsets = [0; consts::HOMA_MAX_BPAGES];
        unsafe {
            let bufs: Vec<u32> = bufs
                .iter()
                .map(|x| x.as_ptr().offset_from(self.buffer.as_ptr()) as u32)
                .collect();
            bpage_offsets[..bufs.len()].copy_from_slice(&bufs);
        }

        let recvmsg_args = types::homa_recvmsg_args {
            id,
            completion_cookie: 0,
            flags: flags.bits(),
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
        let mut length: usize = length.try_into().unwrap();

        let mut iovec = vec![];
        for i in 0..recvmsg_args.num_bpages as usize {
            let size = min(length, consts::HOMA_BPAGE_SIZE);
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

        Ok((
            recvmsg_args.id,
            recvmsg_args.completion_cookie,
            iovec,
            unsafe {
                SockAddr::new(
                    src_addr,
                    size_of::<libc::sockaddr_storage>().try_into().unwrap(),
                )
                .as_socket()
            },
        ))
    }

    pub fn abort(&self, id: u64, error: c_int) -> nix::Result<i32> {
        let mut abort_args = types::homa_abort_args::new(id, error);
        unsafe { types::homa_abort(self.socket.as_raw_fd(), &mut abort_args) }
    }

    pub fn freeze(&self) -> nix::Result<i32> {
        unsafe { types::homa_freeze(self.socket.as_raw_fd()) }
    }
}
