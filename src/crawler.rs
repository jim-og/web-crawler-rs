use crate::{
    html_downloader::HtmlDownloader, html_parser::HtmlParser, printer::Printer,
    types::CrawlerError, url_filter::UrlFilter,
};
use std::sync::Arc;
use tokio::{
    sync::mpsc::{self, Sender},
    time::sleep,
    time::Duration,
};
use url::Url;

pub struct Crawler;

impl Crawler {
    /// Given a seed URL, visit each URL in the same domain.
    pub async fn run(seed: Url) -> Result<(), CrawlerError> {
        let subdomain = seed
            .host_str()
            .ok_or(CrawlerError::SubdomainError { url: seed.clone() })?;

        // Create a channel to schedule URLs. Add the seed URL.
        let (tx, mut rx) = mpsc::channel(100);
        tx.send(seed.clone())
            .await
            .map_err(|_| CrawlerError::NotScheduled { url: seed.clone() })?;

        // Setup components
        let robots_url = seed.join("robots.txt")?;
        let html_downloader = Arc::new(HtmlDownloader::default());
        let robots_txt = html_downloader.fetch(robots_url).await.unwrap();
        let html_parser = Arc::new(HtmlParser::default());
        let url_filter = Arc::new(UrlFilter::new(subdomain.to_string(), robots_txt.body));

        // Event loop
        loop {
            match rx.try_recv() {
                Ok(url) => {
                    Crawler::process(
                        url,
                        html_downloader.clone(),
                        html_parser.clone(),
                        url_filter.clone(),
                        tx.clone(),
                    );
                }
                Err(_) => {
                    // Wait for crawls to complete.
                    sleep(Duration::from_secs(1)).await;
                    if rx.is_empty() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Spawn a task to fetch, parse, and schedule new URLs to be crawled.
    fn process(
        url: Url,
        html_downloader: Arc<HtmlDownloader>,
        html_parser: Arc<HtmlParser>,
        url_filter: Arc<UrlFilter>,
        url_scheduler: Sender<Url>,
    ) {
        tokio::spawn(async move {
            if let Ok(page) = html_downloader.fetch(url.clone()).await {
                if page.status.is_success() {
                    let links = html_parser.parse(page.body);

                    // Print the links found at this URL
                    let _ = Printer::print(std::io::stdout(), url, &links);

                    // Filter links and add them to the scheduler
                    for link in url_filter.filter(links) {
                        let _ = url_scheduler.send(link).await;
                    }
                }
            }
        });
    }
}
