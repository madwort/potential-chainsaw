# Potential Chainsaw

Reimplement jacktrip in Rust

By Tom Ward / Madwort

https://github.com/madwort/potential-chainsaw

## Install & run

```bash
$ cargo build
# Server mode
$ ./target/debug/jacktrip-rust -s
# Client mode
$ ./target/debug/jacktrip-rust -c 127.0.0.1
```

Test/demo app:

```bash
$ cargo build --bin udpserver
$ ./target/debug/udpserver
```

## Dependencies 

[rust-jack](https://github.com/RustAudio/rust-jack)
