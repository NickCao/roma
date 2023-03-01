use roma::{consts, *};
use socket2::Domain;
use std::net::SocketAddr;

fn main() {
    env_logger::init();
    let socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();
    let listen: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    socket.socket.bind(&listen.into()).unwrap();

    let mut bufs = vec![];
    loop {
        let (id, _, nbufs, addr) = socket
            .recv(0, consts::HomaRecvmsgFlags::REQUEST, &bufs)
            .unwrap();
        socket.send(addr.unwrap(), &nbufs, id, 0).unwrap();
        bufs = nbufs;
    }
}
