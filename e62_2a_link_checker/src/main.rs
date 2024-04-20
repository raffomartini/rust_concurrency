use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{Html, Selector};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("request error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("bad http response: {0}")]
    BadResponse(String),
}

#[derive(Debug)]
struct CrawlCommand {
    url: Url,
    extract_links: bool,
}

fn visit_page(client: &Client, command: &CrawlCommand) -> Result<Vec<Url>, Error> {
    println!("Checking {:#}", command.url);
    let response = client.get(command.url.clone()).send()?;
    if !response.status().is_success() {
        return Err(Error::BadResponse(response.status().to_string()));
    }

    let mut link_urls = Vec::new();
    if !command.extract_links {
        return Ok(link_urls);
    }

    let base_url = response.url().to_owned();
    let body_text = response.text()?;
    let document = Html::parse_document(&body_text);

    let selector = Selector::parse("a").unwrap();
    let href_values = document
        .select(&selector)
        .filter_map(|element| element.value().attr("href"));
    for href in href_values {
        match base_url.join(href) {
            Ok(link_url) => {
                link_urls.push(link_url);
            }
            Err(err) => {
                println!("On {base_url:#}: ignored unparsable {href:?}: {err}");
            }
        }
    }
    Ok(link_urls)
}

struct CheckerLogic {
    url_visited: HashSet<String>,
    domain: String,
}

impl CheckerLogic {
    fn new(start_url: &Url) -> CheckerLogic {
        let mut url_visited = HashSet::new();
        let url = start_url.as_str().to_owned();
        let domain = start_url.domain().unwrap_or_default().to_owned();
        url_visited.insert(url);
        CheckerLogic {
            url_visited: url_visited,
            domain: domain,
        }
    }

    fn to_be_visited(&mut self, url: Url) -> bool {
        let domain = url.domain().unwrap_or_default().to_owned();
        let url = url.as_str().to_owned();
        if self.domain == domain && !self.url_visited.contains(&url) {
            self.url_visited.insert(url);
            true
        } else {
            false
        }
    }
}

fn main() {
    // const MAX_THREADS: u8 = 3;
    let start_url = Url::parse("https://www.google.org").unwrap();
    let client = Client::new();
    let mut url_visited = CheckerLogic::new(&start_url);
    let mut url_to_visit = Vec::new();
    url_to_visit.push(start_url);

    while let Some(url) = url_to_visit.pop() {
        let crawl_command = CrawlCommand {
            url: url,
            extract_links: true,
        };
        let result = visit_page(&client, &crawl_command);
        
        match result{
            Ok(links) => {
                for url in links.into_iter(){
                    if url_visited.to_be_visited(url.clone()) {
                        url_to_visit.push(url);
                    }
                }
            }
            Err(err) => {
                // Invalid link, return error.
                println!("Could not extract links: {err:#}");
                return;
            }
        }
    }
    println!("All links are OK.");
}
