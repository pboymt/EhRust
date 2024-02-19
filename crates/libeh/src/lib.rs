pub mod client;
pub mod dto;
pub mod tags;
pub mod url;
mod utils;

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
