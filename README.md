# Actix COAP Server

Actix based [Constrained Application Protocol(CoAP)](https://tools.ietf.org/html/rfc7252) Server.

## Usage

### server

```bash
cargo run
# Started http server: 127.0.0.1:12345
```

### socat client
Copy port provided in server output and run following command to communicate
with the udp server:
```bash
socat - UDP4:localhost:12345
```
