use anyhow::Result;
use exams::{exams_section, Exams};
use futures::StreamExt;
use reqwest_middleware::ClientBuilder;
use std::fmt::Debug;
use std::result::Result::Ok;
use tracing::info;
use voyager::scraper::{ElementRef, Selector};
use voyager::{Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper};

use utils::charset_middleware::HtmlCharsetWindows1252;

mod characteristics;
use characteristics::{characteristics_section, Characteristics};

pub mod utils;

pub mod exams;

use reqwest::Url;

use crate::Record;

pub mod db;

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
    url: CourseUrl,
}

impl Entry {
    fn new(url: CourseUrl) -> Self {
        Entry {
            characteristics: Characteristics::default(),
            exams: Exams::default(),
            url,
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
            Some(state) => match state {
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
                    let mut entry = Entry::new(response.request_url.into());

                    for header in html.select(&self.main_headers_selector) {
                        match header.inner_html().as_str() {
                            "Endereço e Contactos da Instituição" => {
                                institution_contacts_section(header);
                            }
                            "Características do par Instituição/Curso" => {
                                let mut iter = header.next_siblings();
                                entry.characteristics = characteristics_section(&mut iter);
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
            },
            None => {}
        }
        Ok(None)
    }
}

fn institution_contacts_section(element: ElementRef) {}
fn statistics_section(element: ElementRef) {}
fn information_section(element: ElementRef) {}

pub async fn all_courses() -> Result<()> {
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

    while let Some(output) = collector.next().await {
        if let Ok(course) = output {
            dbg!(course);
        }
    }

    Ok(())
}

pub(super) async fn select_courses(courses: impl Iterator<Item = Record>) -> Result<()> {
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

    while let Some(output) = collector.next().await {
        if let Ok(course) = output {
            dbg!(course);
        }
    }

    Ok(())
}
