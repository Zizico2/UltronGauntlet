use anyhow::Result;
use futures::StreamExt;
use reqwest::header::HeaderValue;
use reqwest::{Request, Url};
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use std::result::Result::Ok;
use std::time::Duration;
use task_local_extensions::Extensions;
use tokio::time::timeout;
use voyager::scraper::{ElementRef, Selector};
use voyager::{Collector, Crawler, CrawlerConfig, RequestDelay, Response, Scraper};

mod charset_middleware;
use charset_middleware::HtmlCharsetWindows1252;

struct MyScraper {
    letter_link_selector: Selector,
    course_link_selector: Selector,
    characteristics_selector: Selector,
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
            characteristics_selector: Selector::parse(
                "#caixa-orange > div.inside2 > h2:not([class])",
            )
            .unwrap(),
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

                        if url.query() == Some("letra=Z") {
                            //if true {
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
                MyScraperState::ScrapingCourse => {
                    let courses = html.select(&self.characteristics_selector);
                    for node in courses {
                        let text = node
                            .first_child()
                            .unwrap()
                            .value()
                            .as_text()
                            .unwrap()
                            .to_string();
                        if text == "Características do par Instituição/Curso" {
                            let next_node = node.next_sibling().unwrap();
                            let val = next_node.value().as_text().unwrap().to_string();
                            dbg!(val);
                            let next_node = next_node.next_sibling().unwrap().next_sibling().unwrap();
                            let val = next_node.value().as_text().unwrap().to_string();
                            dbg!(val);
                            let next_node = next_node.next_sibling().unwrap().next_sibling().unwrap();
                            let val = next_node.value().as_text().unwrap().to_string();
                            dbg!(val);
                        }
                    }
                }
            },
            None => {}
        }
        Ok(None)
    }
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
        if let Ok(course) = output {
            //println!("Visited {} at depth: {}", url, depth);
        }
    }

    Ok(())
}
