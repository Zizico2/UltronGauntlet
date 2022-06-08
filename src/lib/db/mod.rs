use diesel::pg::PgConnection;
use diesel::sqlite::SqliteConnection;
use diesel::{prelude::*, query_dsl::LoadQuery};
use dotenv::dotenv;
use std::fmt::Display;
use std::{env, fs};
use thiserror::Error;
use tracing::info;

use diesel::result::Error as DieselError;

mod models;
pub(crate) mod schema;

use crate::lib::db::models::{Exam, NewExam, NewMain, NewMandatoryExam};

use self::models::{Main, MandatoryExam, NewCnaefArea, NewDurationUnit};

pub fn create_duration_unit<'a>(conn: &mut SqliteConnection, unit: &'a str) {
    use schema::duration_units;

    let new_duration_unit = NewDurationUnit { unit };

    let insert_result = diesel::insert_into(duration_units::table)
        .values(&new_duration_unit)
        .execute(conn);

    if let Err(err) = insert_result {
        info!("{}", err);
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
