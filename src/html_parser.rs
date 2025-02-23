use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use url::Url;

#[derive(Default)]
pub struct HtmlParser {
    data_store: HashSet<String>, // TODO move to separate component
}

impl HtmlParser {
    /// Parse the HTML page body and return links (Blocking)
    pub fn parse(&mut self, body: String) -> HashSet<Url> {
        // Check whether this page has been visited before.
        if self.visited_before(&body) {
            println!("visited before"); // TODO
            return HashSet::new();
        }
        println!("not visited before"); // TODO

        // Page hasn't been seen, extract URLs.
        Self::extract_urls(body)
    }

    /// Calculate hash of the body and check whether it has been seen before.
    fn visited_before(&mut self, body: &str) -> bool {
        let body_hash = Self::calculate_hash(body);
        if self.data_store.contains(&body_hash) {
            return true; // TODO better way of returning early?
        }
        // Track this page's hash for future comparisons.
        self.data_store.insert(body_hash);
        false
    }

    fn calculate_hash(body: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(body);
        format!("{:x}", hasher.finalize())
    }

    fn extract_urls(body: String) -> HashSet<Url> {
        let html = Html::parse_document(&body);
        let selector = Selector::parse("a").unwrap();
        html.select(&selector)
            .filter_map(|element| element.value().attr("href"))
            .filter_map(|href| Url::parse(href).ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::HtmlParser;
    use url::Url;

    fn build_html(seed_link: &String) -> String {
        format!(
            r#"
            <html>
                <body>
                    <a href="https://example.com">Example</a>
                    <a href="/relative-path">Relative Link</a>
                    <a href="{}">Seed Link</a>
                    <a>Not the hrefs you're looking for</a>
                </body>
            </html>
        "#,
            seed_link
        )
        .to_string()
    }

    #[tokio::test]
    async fn visited_new() {
        let mut html_parser = HtmlParser::default();
        assert!(html_parser.visited_before(&build_html(&"https://example.com/kolo".to_string())));
        assert!(html_parser.visited_before(&build_html(&"https://example.com/yaya".to_string())));
    }

    #[tokio::test]
    async fn visited_again() {
        let mut html_parser = HtmlParser::default();
        assert!(html_parser.visited_before(&build_html(&"https://example.com/kolo".to_string())));
        assert!(!html_parser.visited_before(&build_html(&"https://example.com/kolo".to_string())));
    }

    #[tokio::test]
    async fn extract_urls() {
        let seed_link = "https://example.com/toure".to_string();
        let urls = HtmlParser::extract_urls(build_html(&seed_link));
        assert!(urls.len() == 2);
        assert!(urls.contains(&Url::parse(&seed_link).unwrap()));
    }

    // TODO
    // #[tokio::test]
    // async fn extract_real_urls() {
    //     let downloader = HtmlDownloader::default();
    //     let start_url = Url::parse("https://monzo.com/").unwrap();
    //     let page = downloader.fetch(start_url).await.unwrap();
    //     let urls = HtmlParser::extract_urls(page.body);
    //     urls.iter().for_each(|link| println!("{}", link));
    // }
}
