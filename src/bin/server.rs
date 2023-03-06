use roma::{consts::HomaRecvmsgFlags, *};
use socket2::Domain;
use std::net::SocketAddr;

fn main() {
    env_logger::init();
    let mut socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();
    let listen: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    socket.socket.bind(&listen.into()).unwrap();

    let mut bufs = vec![0u8; consts::HOMA_MAX_MESSAGE_LENGTH];
    loop {
        let result = socket.recv(0, HomaRecvmsgFlags::REQUEST, &mut bufs);
        match result {
            Ok((id, _, addr)) => {
                socket.send(addr.unwrap(), &bufs, id, 0).unwrap();
            }
            Err(err) => panic!("{}", err),
        }
    }
}
