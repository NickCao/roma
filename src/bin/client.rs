use roma::*;
use socket2::Domain;
use std::{io::IoSlice, net::SocketAddr};

fn main() {
    env_logger::init();
    let socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();
    let dest: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    let mut buf = vec![];
    for i in 90000..100000 {
        let hello = b"hello".repeat(i);
        let homa = b"homa".repeat(i);
        let data = [IoSlice::new(&hello), IoSlice::new(&homa)];
        let id = socket.send(dest, &data, 0, i as u64).unwrap();
        let resp = socket.recv(id, HOMA_RECVMSG_RESPONSE, &buf).unwrap();
        assert_eq!(i as u64, resp.1);
        buf = resp.2;
    }
}
