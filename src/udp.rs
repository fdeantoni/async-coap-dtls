use std::io::Result;
use std::net::SocketAddr;

use std::net::UdpSocket;
use tokio_io::{AsyncRead,AsyncWrite};
use futures::task::Context;
use std::pin::Pin;
use futures::Poll;

#[derive(Debug)]
pub struct UdpChannel {
    socket: UdpSocket,
    remote_addr: SocketAddr,
}

impl AsyncRead for UdpChannel {
    fn poll_read(self: Pin<&mut Self>, _: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
        Poll::from(self.socket.recv(buf))
    }
}

impl AsyncWrite for UdpChannel {
    fn poll_write(self: Pin<&mut Self>, _: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let addr = self.remote_addr;
        Poll::from(self.socket.send_to(buf, addr))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::from(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::from(Ok(()))
    }
}

impl UdpChannel {
    pub fn new(socket: UdpSocket, remote_addr: SocketAddr) -> UdpChannel {

        UdpChannel {
            socket,
            remote_addr
        }
    }
}

