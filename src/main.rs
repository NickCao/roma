use std::{net::SocketAddr, os::fd::AsRawFd, ptr::null_mut};

use libc::c_void;
use roma::homa_set_buf_args;
use socket2::{Domain, Protocol, Socket, Type};

const IPPROTO_HOMA: i32 = 0xFD;
const SO_HOMA_SET_BUF: i32 = 10;
const HOMA_BPAGE_SHIFT: usize = 16;
const HOMA_BPAGE_SIZE: usize = 1 << HOMA_BPAGE_SHIFT;

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
    assert!(!(ret < 0));
}
