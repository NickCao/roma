use roma::*;
use socket2::Domain;
use std::net::SocketAddr;

fn main() {
    let socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();
    let listen: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    socket.socket.bind(&listen.into()).unwrap();
    loop {
        let mut bufs = vec![];
        bufs = socket.recv(0, HOMA_RECVMSG_REQUEST, &bufs).unwrap();
        dbg!(bufs);
    }
}
