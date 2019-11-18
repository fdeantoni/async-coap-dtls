use async_coap::prelude::*;
use async_coap::{RespondableInboundContext, Error, LinkFormatWrite, LINK_ATTR_TITLE};
use core::fmt::Write;
// For `write!()`
use core::borrow::Borrow;
use option::CONTENT_FORMAT;

#[macro_use]
extern crate log;

fn receive_handler<T: RespondableInboundContext>(context: &T) -> Result<(), Error> {
    let msg = context.message();
    let uri = msg.options().extract_uri()?;
    let decoded_path = uri.raw_path().unescape_uri().skip_slashes().to_cow();

    match (msg.msg_code(), decoded_path.borrow()) {
        // Handle GET /test
        (MsgCode::MethodGet, "test") => context.respond(|msg_out| {
            println!("In fetch handler {:?}", msg.payload());
            msg_out.set_msg_code(MsgCode::SuccessContent);
            msg_out.insert_option(CONTENT_FORMAT, ContentFormat::TEXT_PLAIN_UTF8)?;
            write!(msg_out, "Successfully fetched {:?}!", uri.as_str())?;
            Ok(())
        }),

        // Handle GET /.well-known/core, for service discovery.
        (MsgCode::MethodGet, ".well-known/core") => context.respond(|msg_out| {
            msg_out.set_msg_code(MsgCode::SuccessContent);
            msg_out.insert_option(CONTENT_FORMAT, ContentFormat::APPLICATION_LINK_FORMAT)?;
            LinkFormatWrite::new(msg_out)
                .link(uri_ref!("/test"))
                .attr(LINK_ATTR_TITLE, "Test Resource")
                .finish()?;
            Ok(())
        }),

        // Handle unsupported methods
        (_, "test") | (_, ".well-known/core") => context.respond(|msg_out| {
            msg_out.set_msg_code(MsgCode::ClientErrorMethodNotAllowed);
            write!(msg_out, "Method \"{:?}\" Not Allowed", msg.msg_code())?;
            Ok(())
        }),

        // Everything else is a 4.04
        (_, _) => context.respond(|msg_out| {
            msg_out.set_msg_code(MsgCode::ClientErrorNotFound);
            write!(msg_out, "{:?} Not Found", uri.as_str())?;
            Ok(())
        }),
    }
}

use std::sync::Arc;
use futures::{prelude::*, executor::LocalPool, task::LocalSpawnExt};
use async_coap::datagram::DatagramLocalEndpoint;
use std::net::UdpSocket;

use std::io;
use openssl::ssl::{SslAcceptor, SslMethod, SslFiletype};

fn ssl_acceptor(certificate: &str, key: &str) -> Result<SslAcceptor, io::Error> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::dtls())?;
    builder
        .set_private_key_file(key, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(certificate).unwrap();
    let acceptor = builder.build();
    Ok(acceptor)
}

pub mod dtls;
use dtls::acceptor::*;

#[tokio::main]
async fn main() {

    env_logger::init();

    let certificate = String::from("test/cert.pem");
    let key = String::from("test/key.pem");
    let acceptor = ssl_acceptor(&certificate, &key).unwrap();
    let server_socket = DtlsAcceptorSocket::new(UdpSocket::bind("127.0.0.1:10000").unwrap(), acceptor);

    let server_endpoint = Arc::new(DatagramLocalEndpoint::new(server_socket));
    let mut pool = LocalPool::new();

    pool.spawner().spawn_local(server_endpoint
        .clone()
        .receive_loop_arc(receive_handler)
        .map(|_| unreachable!())
    ).unwrap();

    pool.run()
}
