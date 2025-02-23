use crate::crawler_error::CrawlerError;
use tokio::sync::mpsc::{self, Receiver, Sender};
use url::Url;

pub struct UrlScheduler {
    tx: Sender<Url>,
    rx: Receiver<Url>,
}

impl UrlScheduler {
    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer);
        UrlScheduler { tx, rx }
    }

    pub async fn send(&self, url: Url) -> Result<(), CrawlerError> {
        match self.tx.send(url.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("{}", e);
                Err(CrawlerError::NotScheduled { url })
            }
        }
    }

    pub async fn recv(&mut self) -> Option<Url> {
        self.rx.recv().await
    }
}
