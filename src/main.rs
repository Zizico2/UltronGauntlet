use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Ok, Result};
use futures::StreamExt;
use reqwest::Url;
use tokio::time::timeout;
use voyager::scraper::Selector;
use voyager::{Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper};

#[derive(Debug)]
struct CourseInfo {
    id: String,
    url: Url,
}

#[derive(Debug, Default)]
struct OptionalCourseInfo {
    id: Option<String>,
    url: Option<Url>,
}

struct MyScraper {
    course_selector: Selector,
    url_selector: Selector,
    test_sel: Selector,
}
#[derive(Debug)]
struct InstitutionCode(String);
#[derive(Debug)]
struct CourseCode(String);

impl Default for MyScraper {
    fn default() -> Self {
        Self {
            course_selector: Selector::parse(".lin-curso-c3 a").unwrap(),
            url_selector: Selector::parse("div").unwrap(),
            test_sel: Selector::parse("div#caixa-orange > div.inside2").unwrap(),
        }
    }
}

/// The state model
#[derive(Debug)]
enum MyScraperState {
    CoursesByLetter,
    UniqueCourse,
}

#[derive(Debug)]
struct CourseI {
    institution_code: InstitutionCode,
    course_code: CourseCode,
    degree: String,
    cnaef: String,
    duration: String,
    ects: u16,
    r#type: String,
}

impl Scraper for MyScraper {
    type Output = CourseI;

    type State = MyScraperState;

    fn scrape(
        &mut self,
        response: Response<Self::State>,
        crawler: &mut Crawler<Self>,
    ) -> Result<Option<Self::Output>> {
        if !response.response_status.is_success() {
            dbg!("Yikes!");
            return Ok(None);
        }

        let html = response.html();

        let state = match response.state {
            Some(state) => state,
            None => panic!("wtf is goin on"),
        };

        match state {
            MyScraperState::CoursesByLetter => {
                let anchor_array = html.select(&self.course_selector);

                for anchor in anchor_array {
                    let url = format!(
                        "https://www.dges.gov.pt/guias/{}",
                        anchor.value().attr("href").unwrap()
                    );

                    crawler.visit_with_state(&url, MyScraperState::UniqueCourse);
                }

                return Ok(None);
            }
            MyScraperState::UniqueCourse => {
                let a = html.select(&self.test_sel);

                let mut institution_code: Option<InstitutionCode> = None;
                let mut course_code: Option<CourseCode> = None;
                let mut degree: &str = "".into();
                let mut cnaef: &str = "".into();
                let mut duration: &str = "".into();
                let mut ects: u16 = 0;
                let mut r#type: &str = "".into();

                for elem in a {
                    let t = elem.text();
                    for node in t.clone() {
                        if node.contains("C�digo: ") {
                            let mut codes = node.split("C�digo: ").last().unwrap().split("/");
                            institution_code =
                                Some(InstitutionCode(codes.next().unwrap().trim().into()));
                            course_code = Some(CourseCode(codes.last().unwrap().trim().into()));
                        } else if node.contains("Grau: ") {
                            degree = node.split("Grau: ").last().unwrap().trim().into();
                        } else if node.contains("CNAEF: ") {
                            cnaef = node.split("CNAEF: ").last().unwrap().trim().into();
                        } else if node.contains("Dura��o: ") {
                            duration = node.split("Dura��o: ").last().unwrap().trim().into();
                        } else if node.contains("ECTS: ") {
                            ects = node.split("ECTS: ").last().unwrap().trim().parse().unwrap();
                        } else if node.contains("Tipo de Ensino: ") {
                            r#type = node.split("Tipo de Ensino: ").last().unwrap().trim();
                        } else if node.contains("Concurso: ") {
                            //code = node.split("Concurso: ").last().unwrap().trim().into();
                        }
                    }
                }

                return Ok(Some(CourseI {
                    institution_code: institution_code.unwrap(),
                    course_code: course_code.unwrap(),
                    degree: degree.into(),
                    cnaef: cnaef.into(),
                    duration: duration.into(),
                    ects: ects.into(),
                    r#type: r#type.into(),
                }));
            }
        }
    }
}

#[inline]
fn letter_query(c: char) -> String {
    format!("?letra={}", c)
}

#[inline]
fn next_letter(c: char) -> Option<char> {
    let a = 'a' as u32;
    let z = 'z' as u32;
    let c = c as u32 + 1;
    if c > z || c < a {
        return None;
    }

    char::from_u32(c)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = CrawlerConfig::default()
        .respect_robots_txt()
        .allow_domain_with_delay(
            "www.dges.gov.pt",
            RequestDelay::Fixed(std::time::Duration::from_secs(10)),
        )
        .scrape_non_success_response();

    let mut collector = Collector::new(MyScraper::default(), config);

    let mut letter = Some('z');
    loop {
        match letter {
            Some(u) => {
                let d = format!(
                    "https://www.dges.gov.pt/guias/indcurso.asp{}",
                    letter_query(u)
                );
                collector
                    .crawler_mut()
                    .visit_with_state(d.clone(), MyScraperState::CoursesByLetter);

                letter = next_letter(u);
            }
            None => break,
        }
    }

    while let Result::Ok(Some(output)) = timeout(Duration::from_secs(120), collector.next()).await {
        let res = output.unwrap();
        dbg!(res);
    }

    Ok(())
}
