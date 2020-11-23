[![Build Status](https://img.shields.io/circleci/project/github/naia-rs/naia-socket.svg)](https://circleci.com/gh/naia-rs/naia-socket)
[![Latest Version](https://img.shields.io/crates/v/naia-server-socket.svg)](https://crates.io/crates/naia-server-socket)
[![API Documentation](https://docs.rs/naia-server-socket/badge.svg)](https://docs.rs/naia-server-socket)
![](https://tokei.rs/b1/github/naia-rs/naia-socket)
[![Discord chat](https://img.shields.io/discord/764975354913619988.svg?label=discord%20chat)](https://discord.gg/fD6QCtX)
[![MIT/Apache][s3]][l3]

[s3]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[l3]: docs/LICENSE-MIT

# naia-socket

A cross-platform (currently WebAssembly & Linux) Socket API that wraps unreliable & unordered communication, using WebRTC & UDP.

Utilizes Kyren's wonderful [webrtc-unreliable](https://github.com/kyren/webrtc-unreliable)

## Examples

### Server:

To run a UDP server on Linux: (that will be able to communicate with Linux clients)

    1. cd examples/server
    2. cargo run --features "use-udp"

To run a WebRTC server on Linux: (that will be able to communicate with Web clients)

    1. cd examples/server
    2. cargo run --features "use-webrtc"

### Client:

To run a UDP client on Linux: (that will be able to communicate with a UDP server)

    1. cd examples/client
    2. cargo run

To run a WebRTC client on Web using wasm-bindgen: (that will be able to communicate with a WebRTC server)

    1. Enter in your IP Address at the appropriate spot in examples/client/wasm-bindgen/src/app.rs
    2. cd examples/client/wasm-bindgen
    3. npm install              //should only need to do this once to install dependencies
    4. npm run start            //this will open a web browser, and hot reload

To run a WebRTC client on Web using miniquad: (that will be able to communicate with a WebRTC server)

    FORTHCOMING


To simply build these examples instead of running them, substitute the above commands like so:

    `cargo build` for `cargo run`, and

    `npm run build` for `npm run start`
