use async_coap::prelude::*;
use async_coap::datagram::DatagramLocalEndpoint;
use async_coap_tokio::TokioAsyncUdpSocket;
use futures::prelude::*;
use std::sync::Arc;
use tokio::executor::spawn;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};

pub mod dtls;
use dtls::connector::DtlsConnectorSocket;
use async_coap::message::{OwnedImmutableMessage, MessageRead};

fn ssl_connector() -> Result<SslConnector, std::io::Error> {
    let mut builder = SslConnector::builder(SslMethod::dtls())?;
    builder.set_verify(SslVerifyMode::NONE);
    let connector = builder.build();
    Ok(connector)
}

#[tokio::main]
async fn main() {

//    let socket = TokioAsyncUdpSocket::bind("[::]:0").expect("UDP bind failed");
//    let local_endpoint = Arc::new(DatagramLocalEndpoint::new(socket));

    let connector = ssl_connector().unwrap();
    let ssl_socket = DtlsConnectorSocket::bind(std::net::UdpSocket::bind("127.0.0.1:0").unwrap(), connector).unwrap();
    let local_endpoint = Arc::new(DatagramLocalEndpoint::new(ssl_socket));

    spawn(
        local_endpoint
            .clone()
            .receive_loop_arc(null_receiver!())
            .map(|err| panic!("Receive loop terminated: {}", err)),
    );

    // Create a remote endpoint instance to represent the
    // device we wish to interact with.
    let remote_endpoint = local_endpoint
        .remote_endpoint_from_uri(uri!("coap://127.0.0.1:10000"))
        .expect("Unacceptable scheme or authority in URL");

    // Create a future that sends a request to a specific path
    // on the remote endpoint, collecting any blocks in the response
    // and returning `Ok(OwnedImmutableMessage)` upon success.
    let future = remote_endpoint.send_to(
        rel_ref!("test"),
        CoapRequest::get() // This is a CoAP GET request
            .accept(ContentFormat::TEXT_PLAIN_UTF8) // We only want plaintext
            .emit_successful_response(),
    );

    // Wait until we get the result of our request.
    let result: OwnedImmutableMessage = future.await.unwrap();
    let payload = String::from_utf8_lossy(result.payload());

    println!("result: {:?}", payload.trim_end_matches(char::from(0)));
}