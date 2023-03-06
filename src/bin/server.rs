use roma::{consts::HomaRecvmsgFlags, *};
use socket2::Domain;
use std::{io::ErrorKind, net::SocketAddr};

fn main() {
    env_logger::init();
    let socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();
    let listen: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    socket.socket.bind(&listen.into()).unwrap();

    let mut bufs = vec![];
    loop {
        let result = socket.recv(
            0,
            HomaRecvmsgFlags::REQUEST | HomaRecvmsgFlags::NONBLOCKING,
            &bufs,
        );
        match result {
            Ok((id, _, nbufs, addr)) => {
                socket.send(addr.unwrap(), &nbufs, id, 0).unwrap();
                bufs = nbufs;
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => panic!("{}", err),
        }
    }
}
