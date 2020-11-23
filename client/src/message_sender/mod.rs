cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        // WebRTC Sender //
        mod webrtc;
        pub use webrtc::MessageSender;
    }
    else {
        // UDP Sender //
        mod udp;
        pub use udp::MessageSender;
    }
}
