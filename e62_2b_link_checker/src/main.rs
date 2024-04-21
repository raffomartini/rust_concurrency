use reqwest::Client;
use reqwest::Url;
use scraper::{Html, Selector};
use std::collections::HashSet;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
enum Error {
    #[error("request error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("bad http response: {0}")]
    BadResponse(String),
}

#[derive(Debug)]
struct CrawlHistory {
    url_visited: HashSet<String>,
    domain: String,
}

impl CrawlHistory {
    fn new(start_url: &Url) -> CrawlHistory {
        let url_visited = HashSet::new();
        let domain = start_url.domain().unwrap_or_default().to_owned();
        CrawlHistory {
            url_visited: url_visited,
            domain: domain,
        }
    }

    fn to_be_visited(&mut self, url: &Url) -> bool {
        let domain = url.domain().unwrap_or_default().to_owned();
        let url = url.as_str().to_owned();
        if self.domain == domain && !self.url_visited.contains(&url) {
            self.url_visited.insert(url);
            true
        } else {
            false
        }
    }

    fn len(&self) -> usize {
        self.url_visited.len()
    }
}

async fn visit_page(client: &Client, url: &Url) -> Result<Vec<Url>, Error> {
    println!("Checking {:#}", url);
    let response = client.get(url.clone()).send().await?;
    if !response.status().is_success() {
        println!("Error while opening {:#}.", url);
        return Err(Error::BadResponse(response.status().to_string()));
    }
    
    let mut link_urls = Vec::new();
    let base_url = response.url().to_owned();
    let body_text = response.text().await?;
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


#[tokio::main]
async fn main() {
    // const MAX_THREADS: u8 = 3;
    let start_url = Url::parse("https://www.bbc.com").unwrap();
    let client = Client::new();
    let mut history = CrawlHistory::new(&start_url);
    let mut init = Vec::new();
    init.push(start_url);
    let (tx, mut rx) = mpsc::channel(100);
    tx.send(Ok(init)).await.expect("Receiver closed.");

    let mut sent = 1;
    while let Some(ret) = rx.recv().await {
        sent -= 1;
        match ret{
            Ok(url_vec) => {
                for url in url_vec.into_iter() {
                    if history.to_be_visited(&url) {
                        let tx = tx.clone();
                        let client = client.clone();
                        tokio::spawn(async move {
                            tx.send(
                                visit_page(&client, &url).await
                            ).await.unwrap_or_default();
                        });
                        sent += 1;
                    }
                }
            }
            Err(err) => {
                println!("Could not extract links: {err:#}");
                break;
            }
        } 
        if sent == 0 {
            println!("All links are OK.");
            break;
        }
    }
    println!("Visited {} url", history.len());
}
