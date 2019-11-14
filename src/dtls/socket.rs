use openssl::ssl::SslStream;
use std::sync::{Arc, RwLock};
use std::io::{Write, Read};
use futures::{Poll, StreamExt};
use std::net::{SocketAddr, UdpSocket};

use super::channel::UdpChannel;

pub trait DtlsSocket {

    fn get_socket(&self) -> UdpSocket;

    fn get_channel(&self, remote_addr: SocketAddr) -> Arc<RwLock<SslStream<UdpChannel>>>;

    fn send(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, std::io::Error> {
        println!("In connector send....");
        let channel = self.get_channel(addr);
        channel.clone().write().unwrap().write(buf)
    }

    fn receive(&self, buf: &mut [u8]) -> Poll<Result<(usize, SocketAddr, Option<SocketAddr>), std::io::Error>> {
        println!("In connector receive...");
        let mut peek_buf = [0; 10];
        match self.get_socket().peek_from(&mut peek_buf) {
            Ok((_, from)) => {
                let channel = self.get_channel(from);
                let size = channel.clone().write().unwrap().read(buf)?;
                Poll::Ready(Ok((size, from, None)))
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => {
                    Poll::Pending
                }
                _ => Poll::Ready(Err(e)),
            },
        }
    }
}

