#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate log;
#[macro_use]
extern crate schemars;
#[macro_use]
extern crate serde;

pub mod models;
pub mod worker;

pub use smartx_whitedog_common::init;
