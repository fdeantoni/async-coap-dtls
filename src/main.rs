use async_coap::prelude::*;
use async_coap::{RespondableInboundContext, Error, LinkFormatWrite, LINK_ATTR_TITLE};
use core::fmt::Write;
// For `write!()`
use core::borrow::Borrow;
use option::CONTENT_FORMAT;

fn receive_handler<T: RespondableInboundContext>(context: &T) -> Result<(), Error> {
    let msg = context.message();
    let uri = msg.options().extract_uri()?;
    let decoded_path = uri.raw_path().unescape_uri().skip_slashes().to_cow();

    match (msg.msg_code(), decoded_path.borrow()) {
        // Handle GET /test
        (MsgCode::MethodGet, "test") => context.respond(|msg_out| {
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
use async_coap::datagram::{DatagramLocalEndpoint, AllowStdUdpSocket, DatagramSocketTypes};
//use async_coap::null::NullLocalEndpoint;
use async_coap::message::MessageRead;
//use std::borrow::Cow;

pub mod dtls;

#[tokio::main]
async fn main() {
    let socket = AllowStdUdpSocket::bind("127.0.0.1:0").expect("UDP bind failed");
    let local_addr = socket.local_addr().unwrap();
    let local_endpoint = Arc::new(DatagramLocalEndpoint::new(socket));
    let mut pool = LocalPool::new();

    pool.spawner().spawn_local(local_endpoint
        .clone()
        .receive_loop_arc(receive_handler)
        .map(|_| unreachable!())
    ).unwrap();

    let result = pool.run_until(
        local_endpoint.send(
            local_addr,
            CoapRequest::get()       // This is a CoAP GET request
                .uri_host_path(None, rel_ref!("test")) // Add a path to the message
                .emit_any_response(), // Return the first response we get
        )
    );
    println!("result: {:?}", result);
    let result = result.unwrap();
    assert_eq!(result.msg_code(), MsgCode::SuccessContent);
    assert_eq!(result.msg_type(), MsgType::Ack);


    let result = pool.run_until(
        local_endpoint.send(
            local_addr,
            CoapRequest::post()       // This is a CoAP POST request
                .uri_host_path(None, rel_ref!("test")) // Add a path to the message
                .emit_successful_response() // Return the first successful response we get
                .inspect(|cx| {
                    // Inspect here since we currently can't do
                    // a detailed check in the return value.
                    assert_eq!(cx.message().msg_code(), MsgCode::ClientErrorMethodNotAllowed);
                    assert_eq!(cx.message().msg_type(), MsgType::Ack);
                }),
        )
    );
    println!("result: {:?}", result);
    assert_eq!(result.err(), Some(Error::ClientRequestError));

    let result = pool.run_until(
        local_endpoint.send(
            local_addr,
            CoapRequest::get()       // This is a CoAP GET request
                .emit_successful_response() // Return the first successful response we get
                .uri_host_path(None, rel_ref!("/foobar"))
                .inspect(|cx| {
                    // Inspect here since we currently can't do
                    // a detailed check in the return value.
                    assert_eq!(cx.message().msg_code(), MsgCode::ClientErrorNotFound);
                    assert_eq!(cx.message().msg_type(), MsgType::Ack);
                }),
        )
    );
    println!("result: {:?}", result);
    assert_eq!(result.err(), Some(Error::ResourceNotFound));
}
