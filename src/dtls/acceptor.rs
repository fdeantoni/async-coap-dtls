use futures::task::Context;
use futures::{Poll, StreamExt};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::pin::Pin;

use std::collections::HashMap;
use openssl::ssl::{SslStream, SslAcceptor};

use std::result;
use async_coap::datagram::{AsyncDatagramSocket, DatagramSocketTypes, AsyncSendTo, AsyncRecvFrom, MulticastSocket};
use async_coap::{ALL_COAP_DEVICES_HOSTNAME, ToSocketAddrs};
use std::collections::hash_map::Entry;
use std::sync::{Arc, RwLock};

use super::channel::UdpChannel;
use crate::dtls::socket::DtlsSocket;

pub struct DtlsAcceptorSocket {
    local_socket: UdpSocket,
    acceptor: SslAcceptor,
    streams: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<SslStream<UdpChannel>>>>>>
}

impl DtlsAcceptorSocket {

    pub fn bind(local_socket: UdpSocket, acceptor: SslAcceptor) -> std::io::Result<DtlsAcceptorSocket> {

        println!("Creating acceptor dtls socket...");

        Ok(
            DtlsAcceptorSocket {
                local_socket,
                acceptor,
                streams: Arc::new(RwLock::new(HashMap::new()))
            }
        )
    }
}

impl DtlsSocket for DtlsAcceptorSocket {

    fn get_socket(&self) -> UdpSocket {
        self.local_socket.try_clone().unwrap()
    }

    fn get_channel(&self, remote_addr: SocketAddr) -> Arc<RwLock<SslStream<UdpChannel>>> {
        println!("Getting acceptor channel for {:?}", remote_addr);
        match self.streams.write().unwrap().entry(remote_addr.clone()) {
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
}

dtls_socket!(DtlsAcceptorSocket);



