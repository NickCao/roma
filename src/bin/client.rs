use roma::*;
use socket2::Domain;
use std::{io::IoSlice, net::SocketAddr};

fn main() {
    let socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();
    let dest: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    let result = socket
        .send(
            dest,
            &[
                IoSlice::new(b"hello"),
                IoSlice::new(b"homa"),
                IoSlice::new(b"amd"),
                IoSlice::new(b"roma"),
            ],
            0,
            42,
        )
        .unwrap();
    dbg!(result);
}
