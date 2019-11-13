use std::io;
use openssl::ssl::{SslAcceptor, SslConnector, SslMethod, SslVerifyMode, SslFiletype};

fn ssl_connector() -> Result<SslConnector, io::Error> {
    let mut builder = SslConnector::builder(SslMethod::dtls())?;
    builder.set_verify(SslVerifyMode::NONE);
    let connector = builder.build();
    Ok(connector)
}

fn ssl_acceptor(certificate: &str, key: &str) -> Result<SslAcceptor, io::Error> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::dtls())?;
    builder
        .set_private_key_file(key, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(certificate).unwrap();
    let acceptor = builder.build();
    Ok(acceptor)
}

use std::net::UdpSocket;
use tokio::executor::spawn;
use tokio_openssl::*;

use std::thread;
use std::time::Duration;
use tokio_io::{AsyncReadExt, AsyncWriteExt};

pub mod udp;

#[tokio::main]
async fn main() {

    let certificate = String::from("test/cert.pem");
    let key = String::from("test/key.pem");
    let acceptor = ssl_acceptor(&certificate, &key).unwrap();

    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();

    let server_addr = server.local_addr().unwrap();
    let client_addr = client.local_addr().unwrap();

    let server_channel = udp::UdpChannel::new(
        server.try_clone().unwrap(),
        client_addr
    );

    let client_channel = udp::UdpChannel::new(
        client.try_clone().unwrap(),
        server_addr
    );

    spawn(async move {
        //let mut server = acceptor.accept(server_channel).unwrap();
        let mut server = accept(&acceptor, server_channel).await.unwrap();

        let mut count = 0;

        loop {
            let mut buf = [0; 5];

            server.read_exact(&mut buf).await.unwrap();

            let received = std::str::from_utf8(&buf).unwrap();

            println!("{:?} {:?}", count, received);

            count = count + 1;
            thread::sleep(Duration::from_millis(2));
        }
    });

    let connector = ssl_connector().unwrap().configure().unwrap();

    let mut client = connect(connector, "example.net", client_channel).await.unwrap();

    loop {

        let buf = b"hello";
        client.write_all(buf).await.unwrap();

        thread::sleep(Duration::from_millis(30));
    }

}