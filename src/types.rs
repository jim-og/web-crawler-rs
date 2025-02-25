use thiserror::Error;
use url::{ParseError, Url};

#[derive(Error, Debug, PartialEq)]
pub enum CrawlerError {
    #[error("The URL {url:?} could not be scheduled")]
    NotScheduled { url: Url },
    #[error("Unable to parse URL")]
    ParseError(#[from] ParseError),
    #[error("Could not extract the subdomain from the URL {url:?}")]
    SubdomainError { url: Url },
    #[error("please specify a single URL argument")]
    InputMalformed,
}
