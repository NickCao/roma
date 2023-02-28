use libc::{c_void, memset};
use roma::*;
use std::{
    mem::size_of,
    ptr::{addr_of, addr_of_mut, null_mut},
    slice,
};

fn main() {
    let sockfd = homa_socket(libc::AF_INET);
    assert!(sockfd >= 0);

    let listen_addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: 4000u16.to_be(),
        sin_addr: libc::in_addr {
            s_addr: libc::INADDR_LOOPBACK.to_be(),
        },
        sin_zero: [0; 8],
    };

    let result = unsafe {
        libc::bind(
            sockfd,
            addr_of!(listen_addr) as *const libc::sockaddr,
            size_of::<libc::sockaddr_in>() as u32,
        )
    };
    assert_eq!(result, 0);

    let length = 1000 * HOMA_BPAGE_SIZE;
    let start = unsafe {
        libc::mmap(
            0 as *mut c_void,
            length,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            0,
            0,
        )
    };
    assert_ne!(start, libc::MAP_FAILED);
    unsafe { memset(start, 0, length) };

    let args = homa_set_buf_args { start, length };
    let result = unsafe {
        libc::setsockopt(
            sockfd,
            IPPROTO_HOMA,
            SO_HOMA_SET_BUF,
            std::ptr::addr_of!(args) as *const c_void,
            std::mem::size_of::<homa_set_buf_args>() as u32,
        )
    };
    assert!(result >= 0);

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
        let length = unsafe { libc::recvmsg(sockfd, addr_of_mut!(hdr) as *mut libc::msghdr, 0) };
        assert!(length >= 0);
        let mut payload = vec![];
        dbg!(recv_args.num_bpages);
        for page in 0..recv_args.num_bpages as usize {
            dbg!(recv_args.bpage_offsets[page]);
            unsafe {
                payload.extend_from_slice(slice::from_raw_parts(
                    (start as *const u8).offset(recv_args.bpage_offsets[page] as isize),
                    HOMA_BPAGE_SIZE,
                ));
            }
        }
        dbg!(std::str::from_utf8(&payload[..length as usize]));
    }
}
