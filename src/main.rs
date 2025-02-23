use crawler_error::CrawlerError;
use html_downloader::HtmlDownloader;
use html_parser::HtmlParser;
use printer::Printer;
use std::collections::HashSet;
use url::Url;
use url_filter::UrlFilter;
use url_scheduler::UrlScheduler;

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
    let max_depth = 100;

    // UrlScheduler sets a queue to track the URLs which need to be fetched.
    let mut url_scheduler = UrlScheduler::new(100);
    url_scheduler.send(start_url.clone()).await?;

    // HtmlDownloader is used to fetch pages. Pre-fetch robots.txt to setup the UrlFilter.
    let html_downloader = HtmlDownloader::default();
    let robots_txt = html_downloader.fetch(robots_url).await.unwrap(); // TODO

    // HtmlParser checks whether this web page has been before and extracts links.
    let mut html_parser = HtmlParser::default();

    // UrlFilter filters URLs which have been visited before and other criteria.
    let mut url_filter = UrlFilter::new(max_depth, subdomain.to_string(), robots_txt.body);

    while let Some(url) = url_scheduler.recv().await {
        // Fetch the web page and extract links.
        let links = match html_downloader.fetch(url.clone()).await {
            Ok(page) => html_parser.parse(page.body),
            Err(e) => {
                eprintln!("{}", e);
                HashSet::new()
            }
        };

        if !links.is_empty() {
            // Print the links found at this URL
            let _ = Printer::print(std::io::stdout(), url, links.clone()); // TODO avoid clone

            // Filter links and add them to the scheduler
            for link in url_filter.filter(links) {
                url_scheduler.send(link).await?
            }
        }
    }
    Ok(())
}
