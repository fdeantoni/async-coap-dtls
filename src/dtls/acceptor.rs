use futures::task::Context;
use futures::{Poll, StreamExt};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::pin::Pin;

use std::collections::HashMap;
use openssl::ssl::{SslStream, SslAcceptor};

use std::io::{Read, Write};
use std::result;
use async_coap::datagram::{AsyncDatagramSocket, DatagramSocketTypes, AsyncSendTo, AsyncRecvFrom, MulticastSocket};
use async_coap::{ALL_COAP_DEVICES_HOSTNAME, ToSocketAddrs};
use std::collections::hash_map::Entry;
use std::sync::{Arc, RwLock};

use super::channel::UdpChannel;

pub struct DtlsAcceptorSocket {
    local_socket: UdpSocket,
    acceptor: SslAcceptor,
    channels: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<SslStream<UdpChannel>>>>>>
}

impl DtlsAcceptorSocket {

    pub fn bind(local_socket: UdpSocket, acceptor: SslAcceptor) -> std::io::Result<DtlsAcceptorSocket> {

        println!("Creating acceptor dtls socket...");

        Ok(
            DtlsAcceptorSocket {
                local_socket,
                acceptor,
                channels: Arc::new(RwLock::new(HashMap::new()))
            }
        )
    }

    fn get_channel(&self, remote_addr: SocketAddr) -> Arc<RwLock<SslStream<UdpChannel>>> {
        println!("Getting acceptor channel for {:?}", remote_addr);
        match self.channels.write().unwrap().entry(remote_addr.clone()) {
            Entry::Vacant(entry) => {
                let socket = self.local_socket.try_clone().unwrap();
                let channel = UdpChannel::new(socket, remote_addr.clone());
                let stream = Arc::new( RwLock::new(self.acceptor.accept(channel).unwrap()));
                entry.insert(stream).clone()
            }
            // Cache hit - return value
            Entry::Occupied(entry) => {
                entry.get().clone()
            }
        }
    }

    fn send(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, std::io::Error> {
        println!("In acceptor send...");
        let channel = self.get_channel(addr);
        channel.clone().write().unwrap().write(buf)
    }

    fn receive(&self, buf: & mut [u8]) -> Poll<Result<(usize, SocketAddr, Option<SocketAddr>), std::io::Error>> {
        println!("In acceptor receive...");
        let mut peek_buf = [0; 10];
        match self.local_socket.peek_from(&mut peek_buf) {
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

impl Unpin for DtlsAcceptorSocket {}

impl AsyncDatagramSocket for DtlsAcceptorSocket {}

impl DatagramSocketTypes for DtlsAcceptorSocket {
    type SocketAddr = std::net::SocketAddr;
    type Error = std::io::Error;

    fn local_addr(&self) -> result::Result<Self::SocketAddr, Self::Error> {
        self.local_socket.local_addr()
    }

    fn lookup_host(
        host: &str,
        port: u16,
    ) -> result::Result<std::vec::IntoIter<Self::SocketAddr>, Self::Error>
        where
            Self: Sized,
    {
        if host == ALL_COAP_DEVICES_HOSTNAME {
            Ok(vec![
                SocketAddr::V6(SocketAddrV6::new(
                    "FF02:0:0:0:0:0:0:FD".parse().unwrap(),
                    port,
                    0,
                    0,
                )),
                SocketAddr::V4(SocketAddrV4::new("224.0.1.187".parse().unwrap(), port)),
                SocketAddr::V6(SocketAddrV6::new(
                    "FF03:0:0:0:0:0:0:FD".parse().unwrap(),
                    port,
                    0,
                    0,
                )),
            ]
                .into_iter())
        } else {
            (host, port).to_socket_addrs()
        }
    }
}

impl AsyncSendTo for DtlsAcceptorSocket {
    fn poll_send_to<B>(
        self: Pin<&Self>,
        _: &mut Context<'_>,
        buf: &[u8],
        addr: B,
    ) -> Poll<Result<usize, Self::Error>>
        where
            B: ToSocketAddrs<SocketAddr = Self::SocketAddr, Error = Self::Error>,
    {
        if let Some(addr) = addr.to_socket_addrs()?.next() {
            println!("In acceptor poll_send_to {:?}: {:?}", addr, buf);
            match self.send(buf, addr) {
                Ok(written) => Poll::Ready(Ok(written)),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        Poll::Pending
                    } else {
                        Poll::Ready(Err(e))
                    }
                }
            }
        } else {
            Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                "Address lookup failed",
            )))
        }
    }

    fn send_to<B>(& self, buf: &[u8], addr: B) -> Result<usize, Self::Error>
        where
            B: ToSocketAddrs<SocketAddr = Self::SocketAddr, Error = Self::Error>,
    {
        if let Some(addr) = addr.to_socket_addrs()?.next() {
            println!("In acceptor send_to {:?}: {:?}", addr, buf);
            self.send(buf, addr)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                "Address lookup failed",
            ))
        }
    }
}

impl AsyncRecvFrom for DtlsAcceptorSocket {
    fn poll_recv_from(
        self: Pin<&Self>,
        _: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<(usize, Self::SocketAddr, Option<Self::SocketAddr>), Self::Error>> {
        self.receive(buf)
    }
}

impl MulticastSocket for DtlsAcceptorSocket {
    type IpAddr = std::net::IpAddr;

    fn join_multicast<A>(&self, _addr: A) -> Result<(), Self::Error> where
        A: std::convert::Into<Self::IpAddr> {
        unimplemented!()
    }

    fn leave_multicast<A>(&self, _addr: A) -> Result<(), Self::Error> where
        A: std::convert::Into<Self::IpAddr> {
        unimplemented!()
    }
}





