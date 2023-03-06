use roma::{consts::HomaRecvmsgFlags, *};
use socket2::Domain;
use std::net::SocketAddr;

fn main() {
    env_logger::init();
    let mut socket = HomaSocket::new(Domain::IPV4, 1000).unwrap();
    let dest: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    let mut buf = vec![0u8; consts::HOMA_MAX_MESSAGE_LENGTH];
    for i in 100000..200000 {
        let data = b"hello".repeat(i);
        let id = socket.send(dest, &data, 0, 0).unwrap();
        let (_, _, _) = socket
            .recv(id, HomaRecvmsgFlags::RESPONSE, &mut buf)
            .unwrap();
    }
}
