#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

mod opts;
pub mod subcommand;

#[tokio::main]
async fn main() {
    self::opts::Opts::spawn().await
}
