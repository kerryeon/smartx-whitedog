#[cfg(feature = "rocket2")]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;

pub mod models;

#[cfg(feature = "reqwest2")]
pub use ya_gist_common::api;
pub use ya_gist_common::init;
