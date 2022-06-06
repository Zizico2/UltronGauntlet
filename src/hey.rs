#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod lib;
use crate::lib::db::bin::show_duration_units;

fn main() {
    show_duration_units();
}
