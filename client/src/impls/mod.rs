cfg_if! {
    if #[cfg(all(target_arch = "wasm32", feature = "wasm_bindgen"))] {
        mod wasm_bindgen;
        pub use self::wasm_bindgen::message_sender::MessageSender;
        pub use self::wasm_bindgen::client_socket::ClientSocket;
    }
    else if #[cfg(all(target_arch = "wasm32", feature = "miniquad"))] {
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
