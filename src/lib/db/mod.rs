use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::{env, fs};
use tracing::info;

mod models;
mod schema;

use crate::lib::db::models::NewExam;

use self::models::{NewCnaefArea, NewDurationUnit};

pub fn create_duration_unit<'a>(conn: &SqliteConnection, unit: &'a str) {
    use schema::duration_units;

    let new_duration_unit = NewDurationUnit { unit };

    let insert_result = diesel::insert_into(duration_units::table)
        .values(&new_duration_unit)
        .execute(conn);

    if let Err(err) = insert_result {
        info!("{}", err);
    }
}

pub fn create_cnaef_area<'a>(conn: &SqliteConnection, code: &'a str, name: &'a str) {
    use schema::cnaef_areas;

    let new_cnaef_area = NewCnaefArea { code, name };

    let insert_result = diesel::insert_into(cnaef_areas::table)
        .values(&new_cnaef_area)
        .execute(conn);

    if let Err(err) = insert_result {
        info!("{}", err);
    }
}

pub fn create_exam<'a>(conn: &SqliteConnection, code: &'a str, name: &'a str) {
    use schema::exams;

    let new_exam = NewExam { code, name };

    let insert_result = diesel::insert_into(exams::table)
        .values(&new_exam)
        .execute(conn);

    if let Err(err) = insert_result {
        info!("{}", err);
    }
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    fs::remove_file(&database_url).expect(&format!("Couldn't delete {}", database_url));
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
