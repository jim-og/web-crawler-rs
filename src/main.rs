use crawler_error::CrawlerError;
use futures::stream::{FuturesUnordered, StreamExt};
use html_downloader::HtmlDownloader;
use html_parser::HtmlParser;
use printer::Printer;
use std::sync::Arc;
use tokio::sync::mpsc;
use url::Url;
use url_filter::UrlFilter;
// use url_scheduler::UrlScheduler;

mod crawler;
mod crawler_error;
mod html_downloader;
mod html_parser;
mod printer;
mod url_filter;
mod url_scheduler;

#[tokio::main]
async fn main() -> Result<(), CrawlerError> {
    // TODO config
    let start_url = Url::parse("https://monzo.com/")?;
    let subdomain = start_url.host_str().ok_or(CrawlerError::SubdomainError {
        url: start_url.clone(),
    })?;
    let robots_url = start_url.join("robots.txt")?;

    // // UrlScheduler sets a queue to track the URLs which need to be fetched.
    // let mut url_scheduler = UrlScheduler::new(100);
    // url_scheduler.send(start_url.clone()).await?;

    // URL scheduler with an async queue.
    let (tx, mut rx) = mpsc::channel(100);
    tx.send(start_url.clone())
        .await
        .map_err(|_| CrawlerError::NotScheduled {
            url: start_url.clone(),
        })?;

    // HtmlDownloader is used to fetch pages. Pre-fetch robots.txt to setup the UrlFilter.
    let html_downloader = Arc::new(HtmlDownloader::default());
    let robots_txt = html_downloader.fetch(robots_url).await.unwrap(); // TODO

    // HtmlParser checks whether this web page has been before and extracts links.
    let html_parser = Arc::new(HtmlParser::default());

    // UrlFilter filters URLs which have been visited before and other criteria.
    let url_filter = Arc::new(UrlFilter::new(subdomain.to_string(), robots_txt.body));

    let mut tasks = FuturesUnordered::new();

    while let Some(url) = rx.recv().await {
        let html_downloader = html_downloader.clone();
        let html_parser = html_parser.clone();
        let url_filter = url_filter.clone();
        let scheduler = tx.clone();

        // Spawn a task to fetch, parse, and enqueue new URLs
        tasks.push(tokio::spawn(async move {
            println!("process: {}", url.clone());
            if let Ok(page) = html_downloader.fetch(url.clone()).await {
                let links = html_parser.parse(page.body);
                if !links.is_empty() {
                    // Print the links found at this URL
                    let _ = Printer::print(std::io::stdout(), url.clone(), links.clone()); // TODO avoid clone

                    // Filter links and add them to the scheduler
                    for link in url_filter.filter(links) {
                        println!("queue: {}", link);
                        let _ = scheduler.send(link).await; // TODO
                    }
                }
            }
        }));

        // Limit the number of concurrent requests to avoid overloading the server.
        if tasks.len() >= 10 {
            tasks.next().await;
        }
    }

    // Ensure all remaining tasks finish
    while let Some(_) = tasks.next().await {}

    Ok(())
}
