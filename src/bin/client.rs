use libc::c_void;
use roma::*;
use socket2::Domain;
use std::{
    os::fd::AsRawFd,
    ptr::{addr_of, addr_of_mut},
};

fn main() {
    let socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();

    let mut message = b"hello homa".to_vec();
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
        socket.socket.as_raw_fd(),
        message.as_mut_ptr() as *mut c_void,
        message.len(),
        addr_of!(dest_addr) as *const libc::sockaddr_storage,
        addr_of_mut!(id),
        42,
    );
    assert_eq!(result, 0);
    dbg!(id);
}
