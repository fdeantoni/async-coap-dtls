# DTLS COAP Server and Client

An example [Constrained Application Protocol(CoAP)](https://tools.ietf.org/html/rfc7252) Server 
and client using [rust-async-coap](https://github.com/google/rust-async-coap).

## Usage

### server

```bash
$ RUST_LOG=trace cargo +nightly run --bin coap-server
```

### client

```bash
RUST_LOG=trace cargo +nightly run --bin coap-client
```
