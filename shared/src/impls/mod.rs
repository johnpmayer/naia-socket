cfg_if! {
    if #[cfg(all(target_arch = "wasm32", feature = "wbindgen"))] {
        mod wbindgen;
        pub use self::wbindgen::timer::Timer;
        pub use self::wbindgen::random::Random;
        pub use self::wbindgen::instant::Instant;
    }
    else if #[cfg(all(target_arch = "wasm32", feature = "mquad"))] {
        mod mquad;
        pub use self::mquad::random::Random;
        pub use self::mquad::timer::Timer;
        pub use self::mquad::instant::Instant;
    }
    else {
        mod native;
        pub use native::random::Random;
        pub use native::timer::Timer;
        pub use native::instant::Instant;
    }
}
