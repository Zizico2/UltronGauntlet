use crate::lib::db::create_duration_unit;
use crate::Record;
use anyhow::Result;
use characteristics::{characteristics_section, Characteristics};
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use ego_tree::NodeRef;
use exams::{exams_section, Exams};
use futures::StreamExt;
use reqwest::Url;
use reqwest_middleware::ClientBuilder;
use std::fmt::Debug;
use std::result::Result::Ok;
use tracing::info;
use utils::charset_middleware::HtmlCharsetWindows1252;
use voyager::scraper::Node;
use voyager::scraper::{ElementRef, Selector};
use voyager::{Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper};

use self::characteristics::institution;
use self::characteristics::institution::Address;
use self::characteristics::institution::EmailAddress;
use self::characteristics::institution::PhoneNumber;
use self::characteristics::institution::PhoneNumberList;
use self::characteristics::Institution;
use self::db::create_duration;
use self::db::create_institution;
use self::db::create_main;
use self::db::create_mandatory_exam;
use self::db::{create_cnaef_area, create_exam, establish_connection};
use diesel_migrations::MigrationHarness;

mod characteristics;

pub mod db;
pub mod exams;
pub mod utils;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

struct MyScraper {
    letter_link_selector: Selector,
    course_link_selector: Selector,
    main_headers_selector: Selector,
    course_name_selector: Selector,
    institution_name_selector: Selector,
}

impl Default for MyScraper {
    fn default() -> Self {
        Self {
            letter_link_selector: Selector::parse(
                "a[href*=\"indcurso.asp\"][href*=\"?\"][href*=\"letra=\"]",
            )
            .unwrap(),
            course_link_selector: Selector::parse(
                "a[href*=\"detcursopi.asp\"][href*=\"?\"][href*=\"codc=\"][href*=\"code=\"]",
            )
            .unwrap(),
            main_headers_selector: Selector::parse("#caixa-orange > div.inside2 > h2").unwrap(),
            course_name_selector: Selector::parse("#caixa-orange > div.cab1").unwrap(),
            institution_name_selector: Selector::parse("#caixa-orange > div.cab2").unwrap(),
        }
    }
}

/// The state model
#[derive(Debug)]
enum MyScraperState {
    FindingLetters,
    IteratingCourses,
    ScrapingCourse,
}

#[derive(Debug)]
struct Entry {
    characteristics: Characteristics,
    exams: Exams,
}

impl Entry {
    fn new(url: CourseUrl) -> Self {
        Entry {
            characteristics: Characteristics::default(),
            exams: Exams::default(),
        }
    }
}

/* maybe different file */
pub(crate) struct CourseUrl(Url);

impl From<Url> for CourseUrl {
    fn from(value: Url) -> Self {
        CourseUrl(value)
    }
}

impl Debug for CourseUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CourseUrl")
            .field(&self.0.to_string())
            .finish()
    }
}
/* end file */

impl Scraper for MyScraper {
    type Output = Entry;

    type State = MyScraperState;

    fn scrape(
        &mut self,
        mut response: Response<Self::State>,
        crawler: &mut Crawler<Self>,
    ) -> Result<Option<Self::Output>> {
        let html = response.html();
        match response.state {
            Some(state) => {
                match state {
                    MyScraperState::FindingLetters => {
                        let letters = html.select(&self.letter_link_selector);
                        for node in letters {
                            let url = &mut response.response_url;

                            let mut href = node.value().attr("href").unwrap().splitn(2, "?");
                            {
                                let mut path_segments = url.path_segments_mut().unwrap();
                                path_segments.pop();
                                path_segments.push(href.next().unwrap());
                            }
                            url.set_query(href.last());

                            crawler.visit_with_state(url.clone(), MyScraperState::IteratingCourses);
                        }
                    }

                    MyScraperState::IteratingCourses => {
                        let courses = html.select(&self.course_link_selector);
                        for node in courses {
                            let url = &mut response.response_url;

                            let mut href = node.value().attr("href").unwrap().splitn(2, "?");
                            {
                                let mut path_segments = url.path_segments_mut().unwrap();
                                path_segments.pop();
                                path_segments.push(href.next().unwrap());
                            }
                            url.set_query(href.last());
                            crawler.visit_with_state(url.clone(), MyScraperState::ScrapingCourse);
                        }
                    }
                    MyScraperState::ScrapingCourse => {
                        let mut entry = Entry::new(response.request_url.clone().into());
                        let url: String = response.request_url.to_string();
                        dbg!(url);

                        for header in html.select(&self.main_headers_selector) {
                            match header.inner_html().as_str() {
                            "Endereço e Contactos da Instituição" => {
                                let mut iter = header.next_siblings();
                                entry.characteristics.set_institution_meh(institution_contacts_section(&mut iter));
                            }
                            "Características do par Instituição/Curso" => {
                                let mut iter = header.next_siblings();
                                //TODO
                                entry.characteristics.set_most_of_them(characteristics_section(&mut iter));
                            }
                            "Provas de Ingresso" => {
                                let mut iter = header.next_siblings();
                                entry.exams = exams_section(&mut iter);
                            }
                            "Dados Estatísticos de Candidaturas Anteriores" => {
                                statistics_section(header);
                            }
                            "Outras Informações" => {
                                information_section(header);
                            }
                            // useless but known headers
                            "Guia das Provas de Ingresso de 2022 - Detalhe de Curso<br>&nbsp;"
                            | "some more headers"
                            | "some more headerss"
                            | "some more headersss" => {}

                            // unknown headers
                            text => {
                                //TODO: This should store unkown headers somewhere
                                info!("UNKNOWN HEADER: {}", text);
                            }
                        }
                        }

                        entry.characteristics.course.name =
                            match html.select(&self.course_name_selector).next() {
                                Some(element_ref) => match element_ref.first_child() {
                                    Some(node_ref) => match node_ref.value().as_text() {
                                        Some(text) => Some((text as &str).into()),
                                        None => None,
                                    },
                                    None => None,
                                },
                                None => None,
                            };

                        entry.characteristics.institution.name =
                            match html.select(&self.institution_name_selector).next() {
                                Some(element_ref) => match element_ref.first_child() {
                                    Some(node_ref) => match node_ref.value().as_text() {
                                        Some(text) => Some((text as &str).into()),
                                        None => None,
                                    },
                                    None => None,
                                },
                                None => None,
                            };

                        return Ok(Some(entry));
                    }
                }
            }
            None => {}
        }
        Ok(None)
    }
}

fn institution_contacts_section<'a>(
    iter: &mut impl Iterator<Item = NodeRef<'a, Node>>,
) -> Institution {
    let mut institution = Institution::default();
    while let Some(node) = iter.next() {
        match node.value().as_text() {
            Some(text) => match institution.address {
                Some(ref mut address) => {
                    address.push(text.trim().to_string());
                }
                None => {
                    let mut address = Address::default();
                    address.push(text.trim().to_string());
                    institution.address = Some(address);
                }
            },
            None => break,
        }
        // should be a br
        iter.next();
    }

    for node in iter {
        match node.value() {
            Node::Text(text) => {
                if let Some(phone_numbers) = (text as &str).strip_prefix("Tel: ") {
                    let phone_numbers = phone_numbers.split(", ");
                    match institution.phone_numbers {
                        Some(ref mut phone_number_list) => {
                            for phone_number in phone_numbers {
                                let phone_number = remove_whitespace(phone_number);
                                phone_number_list.push(phone_number.trim().to_string());
                            }
                        }
                        None => {
                            let mut phone_number_list = PhoneNumberList::default();
                            for phone_number in phone_numbers {
                                let phone_number = remove_whitespace(phone_number);
                                phone_number_list.push(phone_number.trim().to_string());
                            }
                            institution.phone_numbers = Some(phone_number_list);
                        }
                    }
                } else if let Some(_faxes) = (text as &str).strip_prefix("Fax: ") {
                }
            }
            Node::Element(_element) => {}
            _ => {}
        }
    }
    institution
}
fn statistics_section(element: ElementRef) {}
fn information_section(element: ElementRef) {}

pub async fn all_courses() -> MyCollector {
    tracing_subscriber::fmt::init();

    let config = CrawlerConfig::default()
        .respect_robots_txt()
        .allow_domain_with_delay(
            "dges.gov.pt",
            RequestDelay::Fixed(std::time::Duration::from_secs(10)),
        )
        //.scrape_non_success_response()
        .set_client(
            ClientBuilder::new(reqwest::ClientBuilder::new().build().unwrap())
                .with(HtmlCharsetWindows1252)
                .build(),
        );

    let mut collector = Collector::new(MyScraper::default(), config);
    collector.crawler_mut().visit_with_state(
        "https://dges.gov.pt/guias/indcurso.asp",
        MyScraperState::FindingLetters,
    );

    MyCollector(collector)
}

pub(super) async fn select_courses(courses: impl Iterator<Item = Record>) -> MyCollector {
    tracing_subscriber::fmt::init();

    let config = CrawlerConfig::default()
        .respect_robots_txt()
        .allow_domain_with_delay(
            "dges.gov.pt",
            RequestDelay::Fixed(std::time::Duration::from_secs(10)),
        )
        //.scrape_non_success_response()
        .set_client(
            ClientBuilder::new(reqwest::ClientBuilder::new().build().unwrap())
                .with(HtmlCharsetWindows1252)
                .build(),
        );

    let mut collector = Collector::new(MyScraper::default(), config);

    for Record {
        course_code,
        institution_code,
    } in courses
    {
        let course_code: String = course_code.into();
        let institution_code: String = institution_code.into();
        collector.crawler_mut().visit_with_state(
            format!(
                "https://dges.gov.pt/guias/detcursopi.asp?codc={}&code={}",
                course_code, institution_code
            ),
            MyScraperState::ScrapingCourse,
        );
    }

    MyCollector(collector)
}

pub struct MyCollector(Collector<MyScraper>);

pub async fn handle_results(collector: &mut MyCollector) {
    let collector = &mut collector.0;
    let mut conn = establish_connection();

    //TODO: HANDLE THIS ERROR
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Please migration god, be good!");

    while let Some(output) = collector.next().await {
        if let Ok(course) = output {
            if let (
                Some(code),
                Some(name),
                Some(address),
                Some(phone_numbers),
                //Some(email_addresses),
            ) = (
                course.characteristics.institution.code,
                course.characteristics.institution.name,
                course.characteristics.institution.address,
                course.characteristics.institution.phone_numbers,
                //course.characteristics.institution.email_addresses,
            ) {
                let code: String = code.into();
                let name: String = name.into();
                let address: Vec<String> = address.into();
                if let Ok(institution) = create_institution(
                    &mut conn,
                    &code,
                    &name,
                    address.iter(),
                    phone_numbers.into_iter(),
                    //  email_addresses.into_iter(),
                    vec![""].into_iter(),
                ) {
                    if let Some(ects) = course.characteristics.ects {
                        let ects: u16 = ects.into();
                        let main = create_main(&mut conn, ects as i32, institution.rowid);

                        if let Ok(main) = main {
                            if let Some(name) = course.characteristics.duration.unit {
                                let name: String = name.into();
                                let duration_unit = create_duration_unit(&mut conn, &name);
                                if let Some(ammount) = course.characteristics.duration.ammount {
                                    if let Ok(duration_unit) = duration_unit {
                                        let ammount: u8 = ammount.into();
                                        create_duration(
                                            &mut conn,
                                            main.rowid,
                                            duration_unit.rowid,
                                            ammount as i32,
                                        );
                                    }
                                }
                            }

                            if let Some(code) = course.characteristics.cnaef_area.code {
                                if let Some(name) = course.characteristics.cnaef_area.name {
                                    let code: String = code.into();
                                    let name: String = name.into();
                                    create_cnaef_area(&mut conn, &code, &name);
                                }
                            }

                            if let Some(exams) = course.exams.optional {
                                for exams in exams {
                                    for exam_group in exams {
                                        for exam in exam_group {
                                            if let Some(code) = exam.code {
                                                if let Some(name) = exam.name {
                                                    let code: String = code.into();
                                                    let name: String = name.into();

                                                    create_exam(&mut conn, &code, &name);
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(exams) = course.exams.mandatory {
                                for exam in exams {
                                    if let Some(code) = exam.code {
                                        if let Some(name) = exam.name {
                                            let code: String = code.into();
                                            let name: String = name.into();
                                            if let Ok(exam) = create_exam(&mut conn, &code, &name) {
                                                create_mandatory_exam(
                                                    &mut conn, exam.rowid, main.rowid,
                                                );
                                            } else {
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
