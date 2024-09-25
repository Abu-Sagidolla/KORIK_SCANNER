use futures::stream::StreamExt;
use regex::Regex;
use reqwest;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::sender::suraw::jiber;
use hyper::HeaderMap;


#[derive(Debug)]
pub struct CrawlLink {
    pub inner: Vec<String>,
    pub outer: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ResponseData {
    status_code: String,
    endpoint: String,
    body_length: usize,
}

pub trait Similar {
    fn is_similar_to(&self, other: &Self, tolerance: u32) -> bool;
}

impl Similar for ResponseData {
    fn is_similar_to(&self, other: &ResponseData, tolerance: u32) -> bool {
        let diff = if self.body_length > other.body_length {
            self.body_length - other.body_length
        } else {
            other.body_length - self.body_length
        };
        diff <= tolerance as usize
    }
}

impl CrawlLink {
    pub async fn scrape_all(&self, url: &str) -> Vec<String> {
        if self.inner != vec![""] {
            self.inner.iter().map(|link| format!("{}{}", url.to_string(), link)).collect()
        } else {
            vec!["".to_string()]
        }
    }
    pub fn combine(links: Vec<CrawlLink>) -> CrawlLink {
        let mut combined_inner = Vec::new();
        let mut combined_outer = Vec::new();

        for link in links {
            combined_inner.extend(link.inner);
            combined_outer.extend(link.outer);
        }

        // Return a new CrawlLink with all combined inner and outer links
        CrawlLink {
            inner: combined_inner,
            outer: combined_outer,
        }
    }
    pub async fn depth(&self, url: &str) -> CrawlLink {
        let inners = Arc::new(Mutex::new(Vec::new()));
        let outers = Arc::new(Mutex::new(Vec::new()));

        //firstly get from /robots.txt
        let robots = jiber(&format!("{}/robots.txt", url)).await.unwrap_or(("".to_string(),HeaderMap::new(),"".to_string()))
        ;
        let re = Regex::new(r#"\s*(/[^ \n]*)"#).unwrap();
        let endpoints: Vec<_> = re
            .captures_iter(&robots.0)
            .map(|endpoint| {
                endpoint[1].to_string()
            })
            .collect();
        inners.lock().unwrap().extend(endpoints);

        let _requests = futures::stream::iter(self.scrape_all(url).await.iter().map(|link| {
            let link = link.to_string();
            let inners = Arc::clone(&inners);
            let outers = Arc::clone(&outers);

            tokio::spawn(async move {
                let res = match jiber(&link).await {
                    Ok(res) => res,
                    Err(err) => {
                        return;
                    }
                };
                let parsed_links = parse(&res.0);
                inners.lock().unwrap().extend(parsed_links.inner);
                outers.lock().unwrap().extend(parsed_links.outer);
            })
        }))
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await;

        let inners = Arc::try_unwrap(inners)
            .expect("Failed to unwrap Arc")
            .into_inner()
            .expect("Failed to get inner data from Mutex");
        let outers = Arc::try_unwrap(outers)
            .expect("Failed to unwrap Arc")
            .into_inner()
            .expect("Failed to get inner data from Mutex");


        CrawlLink { inner: inners, outer: outers }
    }

}


pub async fn parse_file(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let line = line?;
        lines.push(line);
    }

    Ok(lines)
}

pub fn remove_duplicates_with_tolerance<T: Eq + std::hash::Hash + Clone + Similar>(
    vec: Vec<T>,
    tolerance: u32,
) -> Vec<T> {
    let mut set: HashSet<T> = HashSet::new();
    let mut vec_without_duplicates = Vec::new();

    for item in vec {
        let mut is_duplicate = false;
        for existing in &set {
            if existing.is_similar_to(&item, tolerance) {
                is_duplicate = true;
                break;
            }
        }
        if !is_duplicate {
            set.insert(item.clone()); 
            vec_without_duplicates.push(item);
        }
    }

    vec_without_duplicates
}


pub fn parse(body: &String) -> CrawlLink {
    if body.is_empty() {
        return CrawlLink {
            inner: vec!["".to_string()],
            outer: vec!["".to_string()],
        };
    }

    let document = Html::parse_document(&body);

    // Selectors for common tags containing URLs
    let a_selector = Selector::parse("a").unwrap();
    let form_selector = Selector::parse("form").unwrap();
    let link_selector = Selector::parse("link").unwrap();
    let script_selector = Selector::parse("script").unwrap();

    let mut inner_links = Vec::new();
    let mut outer_links = Vec::new();

    // Helper function to determine if the link is internal (endpoint)
    let is_internal = |link: &str| -> bool {
        // Internal if it starts with /, ./, ../, or has no protocol (no http:// or https://)
        link.starts_with("/") || link.starts_with("./") || link.starts_with("../") || !(link.starts_with("http://") || link.starts_with("https://"))
    };

    // Collect href attributes from <a> and <link> tags (internal or external)
    document
        .select(&a_selector)
        .filter_map(|element| element.value().attr("href").map(String::from))
        .for_each(|link| {
            if is_internal(&link) {
                inner_links.push(link);
            } else {
                outer_links.push(link);
            }
        });

    document
        .select(&link_selector)
        .filter_map(|element| element.value().attr("href").map(String::from))
        .for_each(|link| {
            if is_internal(&link) {
                inner_links.push(link);
            } else {
                outer_links.push(link);
            }
        });

    // Collect action attributes from <form> tags (internal endpoints for form submissions)
    document
        .select(&form_selector)
        .filter_map(|element| element.value().attr("action").map(String::from))
        .for_each(|link| {
            if is_internal(&link) {
                inner_links.push(link);
            } else {
                outer_links.push(link);
            }
        });

    // Collect src attributes from <script> tags (internal scripts)
    document
        .select(&script_selector)
        .filter_map(|element| element.value().attr("src").map(String::from))
        .for_each(|link| {
            if is_internal(&link) {
                inner_links.push(link);
            } else {
                outer_links.push(link);
            }
        });

    // Return the collected links as CrawlLink
    CrawlLink {
        inner: inner_links,
        outer: outer_links,
    }
}