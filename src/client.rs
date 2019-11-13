use async_coap::prelude::*;
use async_coap::datagram::{DatagramLocalEndpoint, DatagramSocketTypes};
use async_coap_tokio::TokioAsyncUdpSocket;
use futures::prelude::*;
use std::sync::Arc;
use tokio::executor::spawn;
use openssl::ssl::{SslAcceptor, SslConnector, SslMethod, SslVerifyMode, SslFiletype};
use std::net::SocketAddr;
use std::str::FromStr;

pub mod dtls;
use dtls::connector::DtlsConnectorSocket;

fn ssl_connector() -> Result<SslConnector, std::io::Error> {
    let mut builder = SslConnector::builder(SslMethod::dtls())?;
    builder.set_verify(SslVerifyMode::NONE);
    let connector = builder.build();
    Ok(connector)
}

#[tokio::main]
async fn main() {
    let socket = TokioAsyncUdpSocket::bind("[::]:0")
        .expect("UDP bind failed");

    // Create a new local endpoint from the socket we just created,
    // wrapping it in a `Arc<>` to ensure it can live long enough.
    let local_endpoint = Arc::new(DatagramLocalEndpoint::new(socket));

    // Add our local endpoint to the pool, so that it
    // can receive packets.
    spawn(
        local_endpoint
            .clone()
            .receive_loop_arc(null_receiver!())
            .map(|err| panic!("Receive loop terminated: {}", err)),
    );

    let connector = ssl_connector().unwrap();
    let client_socket = DtlsConnectorSocket::bind(std::net::UdpSocket::bind("127.0.0.1:0").unwrap(), connector).unwrap();

    let client_addr = client_socket.local_addr().unwrap();
    let client_endpoint = Arc::new(DatagramLocalEndpoint::new(client_socket));

    let server_address = SocketAddr::from_str("127.0.0.1:10000").unwrap();

    // Create a remote endpoint instance to represent the
    // device we wish to interact with.
    let remote_endpoint = client_endpoint
        .remote_endpoint_from_uri(uri!("coap://127.0.0.1:10000"))
        .expect("Unacceptable scheme or authority in URL");

    // Create a future that sends a request to a specific path
    // on the remote endpoint, collecting any blocks in the response
    // and returning `Ok(OwnedImmutableMessage)` upon success.
    let future = remote_endpoint.send_to(
        rel_ref!("test"),
        CoapRequest::get() // This is a CoAP GET request
            .accept(ContentFormat::TEXT_PLAIN_UTF8) // We only want plaintext
            .emit_any_response(),
//            .block2(Some(Default::default())) // Enable block2 processing
//            .emit_successful_collected_response(), // Collect all blocks
    );

    // Wait until we get the result of our request.
    let result = future.await;

    assert!(result.is_ok(), "Error: {:?}", result.err().unwrap());
}