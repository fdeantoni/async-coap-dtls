use std::net::SocketAddr;

pub mod server;
use server::*;


fn main() -> std::io::Result<()> {
    let sys = actix::System::new("echo-udp");

    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let server = CoapServer::new(addr);
    println!("Started server on: {:?}", server.addr());
    sys.run()
}
