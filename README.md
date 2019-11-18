# Example DTLS COAP Server and Client

An example [Constrained Application Protocol(CoAP)](https://tools.ietf.org/html/rfc7252) Server 
and client using [rust-async-coap](https://github.com/google/rust-async-coap).

## Usage

### server

```bash
$ RUST_LOG=trace cargo +nightly run --bin coap-server
```

### client

```bash
$ RUST_LOG=trace cargo +nightly run --bin coap-client
```

## Considerations

Some things to consider here:
* No clone exists for openssl::ssl::SslStream 
The only way this seemed to work well was to wrap it in an Arc with RwLock.
* DTLS v1.2 has no session ID
A session can only live for the duration of a request. Perhaps with [DTLS v1.3](https://tlswg.org/dtls13-spec/draft-ietf-tls-dtls13.html) 
this will change as this proposal will introduce the concept of a session ID.

 