use libc::c_void;
use roma::*;
use std::{
    mem::size_of,
    ptr::{addr_of, addr_of_mut},
};

fn main() {
    let sockfd = homa_socket(libc::AF_INET);
    assert!(!(sockfd < 0));

    let src_addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: 4001u16.to_be(),
        sin_addr: libc::in_addr {
            s_addr: libc::INADDR_ANY,
        },
        sin_zero: [0; 8],
    };
    let result = unsafe {
        libc::bind(
            sockfd,
            addr_of!(src_addr) as *const libc::sockaddr,
            size_of::<libc::sockaddr_in>() as u32,
        )
    };
    assert_eq!(result, 0);

    let mut message = b"hello homa";
    let mut id = 0;
    let dest_addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: 4000u16.to_be(),
        sin_addr: libc::in_addr {
            s_addr: u32::from_le_bytes([127, 0, 0, 1]),
        },
        sin_zero: [0; 8],
    };
    let result = homa_send(
        sockfd,
        addr_of_mut!(message) as *mut c_void,
        message.len(),
        addr_of!(dest_addr) as *const libc::sockaddr_storage,
        addr_of_mut!(id),
        42,
    );
    assert_eq!(result, 0);
    dbg!(id);
}
