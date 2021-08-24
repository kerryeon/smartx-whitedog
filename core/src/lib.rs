#[macro_use]
extern crate serde;

#[cfg(feature = "reqwest")]
pub mod api;
pub mod models;

pub fn init_logger() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
}
