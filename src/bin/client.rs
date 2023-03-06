use roma::{consts::HomaRecvmsgFlags, *};
use socket2::Domain;
use std::{io::IoSlice, net::SocketAddr};

fn main() {
    env_logger::init();
    let socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();
    let dest: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    let mut buf = vec![];
    for i in 100000..200000 {
        let hello = b"hello".repeat(i);
        let homa = b"homa".repeat(i);
        let data = [IoSlice::new(&hello), IoSlice::new(&homa)];
        let id = socket.send(dest, &data, 0, 0).unwrap();
        let (_, _, nbufs, _) = socket.recv(id, HomaRecvmsgFlags::RESPONSE, &buf).unwrap();
        buf = nbufs;
    }
}
