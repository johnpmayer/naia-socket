cfg_if! {
    if #[cfg(all(target_arch = "wasm32", feature = "wbindgen"))] {
        mod wasm_bindgen;
        pub use self::wasm_bindgen::message_sender::MessageSender;
        pub use self::wasm_bindgen::client_socket::ClientSocket;
    }
    else if #[cfg(all(target_arch = "wasm32", feature = "mquad"))] {
        mod miniquad;
        pub use self::miniquad::message_sender::MessageSender;
        pub use self::miniquad::client_socket::ClientSocket;
    }
    else {
        mod udp;
        pub use udp::message_sender::MessageSender;
        pub use udp::client_socket::ClientSocket;
    }
}
