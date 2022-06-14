use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::fmt::Display;
use std::{env, fs};
use thiserror::Error;
use tracing::info;

use diesel::result::Error as DieselError;

mod models;
pub(crate) mod schema;

use crate::lib::db::models::{Exam, NewDuration, NewExam, NewMain, NewMandatoryExam};

use self::models::{DurationUnit, Main, MandatoryExam, NewCnaefArea, NewDurationUnit};

pub fn create_duration(
    conn: &mut SqliteConnection,
    main_id: i32,
    duration_unit_id: i32,
    ammount: i32,
) {
    use schema::durations;

    let new_duration = NewDuration {
        rowid: main_id,
        unit: duration_unit_id,
        ammount,
    };

    let insert_result = diesel::insert_into(durations::table)
        .values(&new_duration)
        .execute(conn);

    if let Err(err) = insert_result {
        info!("{}", err);
    }
}

pub fn create_duration_unit<'a>(
    conn: &mut SqliteConnection,
    new_name: &'a str,
) -> Result<DurationUnit, ()> {
    use schema::duration_units;
    use schema::duration_units::dsl::*;

    let new_duration_unit = NewDurationUnit { name: new_name };

    /*
    let result = diesel::replace_into(duration_units::table)
        .values(&new_duration_unit)
        .get_result(conn);
    */

    let result = conn.transaction(|conn| {
        let mut result = diesel::insert_into(duration_units::table)
            .values(&new_duration_unit)
            .get_result::<DurationUnit>(conn);
        if let Err(ref err) = result {
            match err {
                DieselError::DatabaseError(err, _) => match err {
                    diesel::result::DatabaseErrorKind::UniqueViolation => {
                        result = duration_units
                            .filter(name.eq(new_name))
                            .first::<DurationUnit>(conn);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        result
    });

    match result {
        Ok(result) => Ok(result),
        Err(_) => Err(()),
    }
}

pub fn create_cnaef_area<'a>(conn: &mut SqliteConnection, code: &'a str, name: &'a str) {
    use schema::cnaef_areas;

    let new_cnaef_area = NewCnaefArea { code, name };

    let insert_result = diesel::insert_into(cnaef_areas::table)
        .values(&new_cnaef_area)
        .execute(conn);

    if let Err(err) = insert_result {
        info!("{}", err);
    }
}

#[derive(Error, Debug)]
pub enum CreateExamError {
    Error(DieselError),
}
impl Display for CreateExamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateExamError::Error(diesel_error) => {
                write!(f, "{}", diesel_error)
            }
        }
    }
}

//TODO: take a look
pub fn create_exam(
    conn: &mut SqliteConnection,
    code_val: &str,
    name_val: &str,
) -> Result<Exam, CreateExamError> {
    use schema::exams;
    use schema::exams::dsl::*;

    let new_exam = NewExam {
        code: code_val,
        name: name_val,
    };

    let result = conn.transaction(|conn| {
        let mut result = diesel::insert_into(exams::table)
            .values(&new_exam)
            .get_result::<Exam>(conn);
        if let Err(ref err) = result {
            match err {
                DieselError::DatabaseError(err, _) => match err {
                    diesel::result::DatabaseErrorKind::UniqueViolation => {
                        result = exams.filter(code.eq(code_val)).first::<Exam>(conn);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        result
    });

    match result {
        Ok(result) => Ok(result),
        Err(err) => Err(CreateExamError::Error(err)),
    }
}

pub fn create_mandatory_exam(
    conn: &mut SqliteConnection,
    exam: i32,
    main: i32,
) -> Result<MandatoryExam, ()> {
    use schema::mandatory_exams;

    let new_exam = NewMandatoryExam { exam, main };

    let result = diesel::insert_into(mandatory_exams::table)
        .values(&new_exam)
        .get_result::<MandatoryExam>(conn);

    match result {
        Ok(result) => Ok(result),
        Err(_) => Err(()),
    }
}

pub fn create_main(conn: &mut SqliteConnection, ects: i32) -> Result<Main, ()> {
    use schema::main;

    let new_main = NewMain { ects };
    let result = diesel::insert_into(main::table)
        .values(&new_main)
        .get_result::<Main>(conn);
    match result {
        Ok(result) => Ok(result),
        Err(_) => Err(()),
    }
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    fs::remove_file(&database_url).ok();
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
