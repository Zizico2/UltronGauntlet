#[macro_use]
extern crate diesel;
extern crate dotenv;


pub mod lib;
use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
};

use std::error::Error;

use serde::Deserialize;

use anyhow::Result;
use clap::Parser;
use lib::{all_courses, select_courses};

#[derive(Debug, Deserialize)]
struct Record {
    institution_code: String,
    course_code: String,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sets a custom source file
    #[clap(short, long, value_name = "FILE", validator = csv_file_exists)]
    source: Option<PathBuf>,
}

fn csv_file_exists(value: &str) -> Result<(), String> {
    let path = Path::new(value);
    let file_exists = path.exists();
    let extension = path.extension().and_then(OsStr::to_str);
    if !file_exists {
        return Err(format!("source file doesn't exist"));
    };
    if extension != Some("csv") {
        //TODO: actually check if the file is csv compliant. behind a flag, maybe cuz performance
        //TODO: maybe even allow a schema to be provided
        return Err(format!("source file isn't a csv"));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    return if let Some(source) = args.source {
        select_courses(read_courses(source).unwrap().into_iter()).await
    } else {
        all_courses().await
    };
}

fn read_courses(buf: PathBuf) -> Result<Vec<Record>, Box<dyn Error>> {
    let f = File::open(buf)?;
    let mut rdr = csv::Reader::from_reader(f);
    let mut res = Vec::new();
    for result in rdr.deserialize() {
        res.push(result?);
    }
    Ok(res)
}
