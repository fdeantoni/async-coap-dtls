use actix::{Actor, Context, AsyncContext, Message, StreamHandler, Addr};
use bytes::BytesMut;
use futures::stream::SplitSink;
use futures::{Stream, Future, Sink};
use std::net::SocketAddr;
use tokio::codec::BytesCodec;
use tokio::net::{UdpFramed, UdpSocket};


#[derive(Message)]
pub struct UdpPacket(pub BytesMut, pub SocketAddr);

pub struct UdpActor {
    pub sink: SplitSink<UdpFramed<BytesCodec>>
}
impl Actor for UdpActor {
    type Context = Context<Self>;
}
impl StreamHandler<UdpPacket, std::io::Error> for UdpActor {
    fn handle(&mut self, msg: UdpPacket, _: &mut Context<Self>) {
        println!("Received: ({:?}, {:?})", msg.0, msg.1);
        (&mut self.sink).send(("PING\n".into(), msg.1)).wait().unwrap();
    }
}

pub struct CoapServer {
    address: SocketAddr,
    actor: Addr<UdpActor>
}

impl CoapServer {
    pub fn new(addr: SocketAddr) -> CoapServer {
        let socket = UdpSocket::bind(&addr).unwrap();
        let address = socket.local_addr().unwrap();
        let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();
        let actor = UdpActor::create(|ctx| {
            ctx.add_stream(stream.map(|(data, sender)| UdpPacket(data, sender)));
            UdpActor{sink}
        });

        CoapServer {
            address,
            actor
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.address
    }
}

