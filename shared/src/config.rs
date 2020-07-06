use std::{default::Default, time::Duration};

/// A Config object required to initialize a given Server/Client Socket,
/// currently unused
#[derive(Clone, Debug)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
