cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        // WasmBindgen WebRTC Sender //
        mod wasm_bindgen;
        pub use self::wasm_bindgen::MessageSender;
    }
    else {
        // UDP Sender //
        mod udp;
        pub use udp::MessageSender;
    }
}
