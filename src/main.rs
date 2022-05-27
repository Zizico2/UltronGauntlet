use anyhow::Result;
use futures::StreamExt;
use reqwest_middleware::ClientBuilder;
use std::result::Result::Ok;
use voyager::scraper::{ElementRef, Selector};
use voyager::{Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper};

mod charset_middleware;
use charset_middleware::HtmlCharsetWindows1252;

struct MyScraper {
    letter_link_selector: Selector,
    course_link_selector: Selector,
    main_headers_selector: Selector,
    course_name_selector: Selector,
    institution_name_selector: Selector,
}
#[derive(Debug)]
struct InstitutionCode(String);
#[derive(Debug)]
struct CourseCode(String);

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
struct CourseI {}

impl Scraper for MyScraper {
    type Output = CourseI;

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

                        //if true {
                        if url.query() == Some("letra=Z") {
                            crawler.visit_with_state(url.clone(), MyScraperState::IteratingCourses);
                        }
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

                // "Características do par Instituição/Curso"
                MyScraperState::ScrapingCourse => {
                    let course_name = html
                        .select(&self.course_name_selector)
                        .next()
                        .unwrap()
                        .inner_html();
                    let institution_name = html
                        .select(&self.institution_name_selector)
                        .next()
                        .unwrap()
                        .inner_html();
                    dbg!(course_name);
                    dbg!(institution_name);
                    for header in html.select(&self.main_headers_selector) {
                        match header.inner_html().as_str() {
                            "Endereço e Contactos da Instituição" => {
                                institution_contacts_section(header);
                            }
                            "Características do par Instituição/Curso" => {
                                characteristics_section(header);
                            }
                            "Provas de Ingresso" => {
                                exams_section(header);
                            }
                            "Dados Estatísticos de Candidaturas Anteriores" => {
                                statistics_section(header);
                            }
                            "Outras Informações" => {
                                information_section(header);
                            }
                            // useless but known headers
                            "Guia das Provas de Ingresso de 2022 - Detalhe de Curso<br>&nbsp;" => {}

                            // unknown headers
                            _ => {
                                println!("UNKNOWN HEADER: {}", header.html())
                            }
                        }
                    }
                }
            },
            None => {}
        }
        Ok(None)
    }
}

fn institution_contacts_section(element: ElementRef) {
    dbg!(element.html());
}

fn characteristics_section(element: ElementRef) {
    let mut it = element.next_siblings();
    while let Some(sibling) = it.next() {
        if let Some(text) = sibling.value().as_text() {
            dbg!(text);
        }
        it.next();
    }
}

fn exams_section(element: ElementRef) {
    dbg!(element.html());
}

fn statistics_section(element: ElementRef) {
    dbg!(element.html());
}
fn information_section(element: ElementRef) {
    dbg!(element.html());
}

#[tokio::main]
async fn main() -> Result<()> {
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
        if let Ok(_course) = output {
            //println!("Visited {} at depth: {}", url, depth);
        }
    }

    Ok(())
}
