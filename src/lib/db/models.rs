use super::schema::{cnaef_areas, duration_units, exams};

#[derive(Insertable)]
#[table_name = "duration_units"]
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
#[table_name = "cnaef_areas"]
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

#[derive(Insertable)]
#[table_name = "exams"]
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
