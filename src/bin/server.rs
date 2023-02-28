use libc::c_void;
use roma::*;
use socket2::{Domain, Protocol, Socket, Type};
use std::{ffi::c_int, net::SocketAddr, os::fd::AsRawFd, ptr::null_mut};

fn main() {
    let socket = Socket::new(
        Domain::IPV4,
        Type::DGRAM,
        Some(Protocol::from(IPPROTO_HOMA)),
    )
    .unwrap();
    socket
        .bind(&"127.0.0.1:4000".parse::<SocketAddr>().unwrap().into())
        .unwrap();

    let length = 1000 * HOMA_BPAGE_SIZE;
    let start = unsafe {
        libc::mmap(
            null_mut() as *mut c_void,
            length,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            0,
            0,
        )
    };
    assert_ne!(start, libc::MAP_FAILED);

    let args = homa_set_buf_args { start, length };
    let ret = unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            IPPROTO_HOMA,
            SO_HOMA_SET_BUF,
            std::ptr::addr_of!(args) as *const c_void,
            std::mem::size_of::<homa_set_buf_args>() as u32,
        )
    };
    assert!(ret >= 0);

    let mut source: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
    let mut recv_args: homa_recvmsg_args = unsafe { std::mem::zeroed() };

    let mut hdr = libc::msghdr {
        msg_name: std::ptr::addr_of_mut!(source) as *mut c_void,
        msg_namelen: std::mem::size_of::<libc::sockaddr_storage>() as u32,
        msg_iov: std::ptr::null_mut() as *mut libc::iovec,
        msg_iovlen: 0,
        msg_control: std::ptr::addr_of_mut!(recv_args) as *mut c_void,
        msg_controllen: std::mem::size_of::<homa_recvmsg_args>(),
        msg_flags: 0,
    };

    loop {
        recv_args.id = 0;
        recv_args.flags = HOMA_RECVMSG_REQUEST;
        let length = unsafe {
            libc::recvmsg(
                socket.as_raw_fd(),
                std::ptr::addr_of_mut!(hdr) as *mut libc::msghdr,
                0,
            )
        };
        assert!(length >= 0);
        let resp_length = unsafe {
            *(start.offset(recv_args.bpage_offsets[0] as isize) as *const c_int).offset(1)
        };
        dbg!(resp_length);
    }
}
