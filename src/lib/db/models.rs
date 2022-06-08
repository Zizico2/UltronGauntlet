use super::schema::{cnaef_areas, duration_units, exams, main, mandatory_exams};
use diesel::AsChangeset;

#[derive(Insertable)]
#[diesel(table_name = duration_units)]
pub struct NewDurationUnit<'a> {
    pub unit: &'a str,
}

#[derive(Queryable)]
pub struct DurationUnit {
    pub rowid: i32,
    pub unit: String,
}

//---------------

#[derive(Insertable)]
#[diesel(table_name = cnaef_areas)]
pub struct NewCnaefArea<'a> {
    pub code: &'a str,
    pub name: &'a str,
}

#[derive(Queryable)]
pub struct CnaefArea {
    pub rowid: i32,
    pub code: String,
    pub name: String,
}

//---------------

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = exams)]
pub struct NewExam<'a> {
    pub code: &'a str,
    pub name: &'a str,
}

#[derive(Queryable)]
pub struct Exam {
    pub rowid: i32,
    pub code: String,
    pub name: String,
}

// main

#[derive(Insertable)]
#[diesel(table_name = main)]
pub struct NewMain {
    pub ects: i32,
}

#[derive(Queryable)]
pub struct Main {
    pub rowid: i32,
    pub ects: i32,
}

// mandatory exams

#[derive(Insertable)]
#[diesel(table_name = mandatory_exams)]
pub struct NewMandatoryExam {
    pub exam: i32,
    pub main: i32,
}

#[derive(Queryable)]
pub struct MandatoryExam {
    pub rowid: i32,
    pub exam: i32,
    pub main: i32,
}
