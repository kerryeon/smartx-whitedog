#[cfg(feature = "rocket2")]
extern crate rocket;
#[macro_use]
extern crate serde;

pub mod models;

#[cfg(all(feature = "api", feature = "reqwest2"))]
pub use ya_gist_common::api;
pub use ya_gist_common::init;
