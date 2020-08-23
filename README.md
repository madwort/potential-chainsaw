# Potential Chainsaw

Reimplement jacktrip in Rust

By Tom Ward / Madwort

https://github.com/madwort/potential-chainsaw

## Install & run

### JackTrip client

This can connect to/from an actual JackTrip server/client.

```bash
$ cargo build --bin jacktrip_client
# Server mode
$ ./target/debug/jacktrip_client -s
# Client mode
$ ./target/debug/jacktrip_client -c 127.0.0.1
```

### Udpserver

This will start a JackTrip server, if you connect to it with a JackTrip client, it will dump the audio data sent to it to the console as a big list of numbers.

```bash
$ cargo build --bin udpserver
$ ./target/debug/udpserver
```

## Dependencies 

[rust-jack](https://github.com/RustAudio/rust-jack)
