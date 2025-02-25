use crawler::Crawler;
use std::env;
use types::CrawlerError;
use url::Url;

mod crawler;
mod html_downloader;
mod html_parser;
mod printer;
mod store;
mod types;
mod url_filter;

#[tokio::main]
async fn main() -> Result<(), CrawlerError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("{}", CrawlerError::InputMalformed);
        return Err(CrawlerError::InputMalformed);
    }

    let seed_url = Url::parse(&args[1].clone())?;
    Crawler::run(seed_url).await
}
