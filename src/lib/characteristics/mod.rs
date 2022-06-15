pub(crate) use contest::Contest;
pub(crate) use course::Course;
pub(crate) use degree::Degree;
pub(crate) use duration::Duration;
pub(crate) use ects::Ects;
pub(crate) use education_type::EducationType;
use ego_tree::NodeRef;
pub(crate) use institution::Institution;
use tracing::info;
use voyager::scraper::Node;
mod cnaef_area;
mod contest;
pub mod course;
mod degree;
mod duration;
mod ects;
mod education_type;
pub mod institution;
use anyhow::{bail, Result};
use cnaef_area::CnaefArea;

#[derive(Debug, Default)]
pub(crate) struct Characteristics {
    pub(crate) course: Course,
    pub(crate) institution: Institution,
    pub(crate) degree: Option<Degree>,
    pub(crate) cnaef_area: CnaefArea,
    pub(crate) duration: Duration,
    pub(crate) ects: Option<Ects>,
    pub(crate) education_type: Option<EducationType>,
    pub(crate) contest: Option<Contest>,
}

impl Characteristics {
    //TODO
    pub fn set_institution_meh(&mut self, institution: Institution) {
        self.institution.address = institution.address;
        self.institution.phone_numbers = institution.phone_numbers;
        self.institution.email_addresses = institution.email_addresses;
    }

    //TODO
    pub fn set_most_of_them(&mut self, chars: Characteristics) {
        self.course = chars.course;
        self.institution.code = chars.institution.code;
        self.institution.name = chars.institution.name;
        self.degree = chars.degree;
        self.cnaef_area = chars.cnaef_area;
        self.duration = chars.duration;
        self.ects = chars.ects;
        self.education_type = chars.education_type;
        self.contest = chars.contest;
    }
}

pub(crate) fn characteristics_section<'a>(
    it: &mut impl Iterator<Item = NodeRef<'a, Node>>,
) -> Characteristics {
    let mut characteristics = Characteristics::default();

    while let Some(sibling) = it.next() {
        if let Some(text) = sibling.value().as_text() {
            if let Some((field, value)) = text.split_once(": ") {
                /*
                TODO: the errors that are being ignored below
                TODO: should, maybe be stored somewhere, or logged at least
                */
                match field {
                    "Código" => {
                        if let Ok((institution_code, course_code)) = parse_code(value) {
                            characteristics.institution.code = Some(institution_code);
                            characteristics.course.code = Some(course_code);
                        }
                    }
                    "Grau" => {
                        characteristics.degree = parse_degree(value).ok();
                    }
                    "Área CNAEF" => {
                        if let Ok(cnaef_area) = parse_cnaef_area(value) {
                            characteristics.cnaef_area = cnaef_area
                        }
                    }
                    "Duração" => {
                        if let Ok(duration) = parse_duration(value) {
                            characteristics.duration = duration
                        }
                    }
                    "ECTS" => {
                        characteristics.ects = parse_ects(value).ok();
                    }
                    "Tipo de Ensino" => {
                        characteristics.education_type = parse_education_type(value).ok();
                    }
                    "Concurso" => {
                        characteristics.contest = parse_contest(value).ok();
                    }
                    field => {
                        //TODO: This should store unkown fields somewhere
                        info!("UNKNOWN FIELD: {}", field);
                    }
                };
            } else {
                break;
            }
        } else {
            break;
        }
        // this should be a <br>. Maybe check this?
        let _br = it.next();
    }
    characteristics
}

fn parse_code(value: &str) -> Result<(institution::Code, course::Code)> {
    let split_value = value.split_once("/");
    if let Some((institution_code, course_code)) = split_value {
        Ok((institution_code.into(), course_code.into()))
    } else {
        bail!("Bad Code string: \"{}\"", value)
    }
}

fn parse_cnaef_area(value: &str) -> Result<CnaefArea> {
    let split_value = value.split_once(" ");
    if let Some((cnaef_area_code, cnaef_area_name)) = split_value {
        let code = Some(cnaef_area_code.into());
        let name = Some(cnaef_area_name.into());
        Ok(CnaefArea { code, name })
    } else {
        bail!("Bad CNAEF Area string: \"{}\"", value)
    }
}

fn parse_degree(value: &str) -> Result<Degree> {
    Ok(value.into())
}

fn parse_duration(value: &str) -> Result<Duration> {
    let split_value = value.split_once(" ");
    match split_value {
        Some((duration_ammount, duration_unit)) => match duration_ammount.parse::<u8>() {
            Ok(duration_ammount) => {
                let ammount = Some(duration_ammount.into());
                let unit = Some(duration_unit.into());
                Ok(Duration { ammount, unit })
            }
            Err(_) => bail!("Bad Duration string: \"{}\"", value),
        },
        None => bail!("Bad Duration string: \"{}\"", value),
    }
}

fn parse_contest(value: &str) -> Result<Contest> {
    Ok(value.into())
}

fn parse_education_type(value: &str) -> Result<EducationType> {
    Ok(value.into())
}

fn parse_ects(value: &str) -> Result<Ects> {
    match value.parse::<u16>() {
        Ok(value) => Ok(value.into()),
        Err(_) => bail!("Bad ECTS string: \"{}\"", value),
    }
}
