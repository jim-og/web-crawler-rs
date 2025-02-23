use robotstxt::DefaultMatcher;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use url::Url;

#[derive(Default)]
pub struct UrlFilter {
    _max_depth: usize, // TODO
    subdomain: String,
    data_store: Arc<Mutex<HashSet<Url>>>,
    robots_txt: String,
}

impl UrlFilter {
    pub fn new(max_depth: usize, subdomain: String, robots_txt: String) -> Self {
        UrlFilter {
            _max_depth: max_depth,
            subdomain,
            data_store: Arc::new(Mutex::new(HashSet::new())),
            robots_txt,
        }
    }

    pub fn filter(&self, urls: HashSet<Url>) -> HashSet<Url> {
        // Exclude certain content types
        // Exclude file extensions
        // Exclude error links
        // Exclude links not in this subdomain
        // Exclude Robots.txt disallowed. Better doing at HtmlDownloader level.
        // Exclude long URL spider traps
        let filtered: Vec<Url> = urls
            .into_iter()
            // Exclude URLs which do not match the subdomain.
            .filter(|url| url.host_str().unwrap_or("") == self.subdomain)
            // Exclude URLs which are not allowed by robots.txt
            .filter(|url| self.allowed(url))
            .collect();

        let mut data_store = match self.data_store.lock().ok() {
            Some(store) => store,
            // If the lock is poisoned do not continue processing.
            None => {
                eprintln!("UrlFilter data store lock poisioned");
                return HashSet::new();
            }
        };

        filtered
            .into_iter()
            // Exclude URLs which have been seen before, add new URLs to data store.
            .filter_map(|url| {
                if data_store.insert(url.clone()) {
                    Some(url) // New, keep
                } else {
                    None // Already seen, skip
                }
            })
            .collect()
    }

    fn allowed(&self, url: &Url) -> bool {
        let mut matcher = DefaultMatcher::default();
        matcher.one_agent_allowed_by_robots(&self.robots_txt, "*", url.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::url_filter::UrlFilter;
    use std::collections::HashSet;
    use url::Url;

    fn build_urls() -> HashSet<Url> {
        // Dataset contains 21 links with a matching subdomain.
        let links = vec![
            "https://we83.adj.st/home?adj_t=1dj2rkno_1dxkjz95&adj_redirect=https%3A%2F%2Fmonzo.com%2Fsign-up&adj_engagement_type=fallback_click",
            "https://uk.trustpilot.com/review/www.monzo.com",
            "https://we83.adj.st/home?adj_t=1dj2rkno&adj_fallback=https%3A%2F%2Fmonzo.com%2Fdownload&adj_engagement_type=fallback_click",
            "https://app.adjust.com/1dxkjz95?fallback=https%3A%2F%2Fmonzo.com%2Fdownload&engagement_type=fallback_click",
            "https://www.psr.org.uk/app-fraud-data",
            "https://we83.adj.st/home?adj_t=1dj2rkno&adj_redirect=https%3A%2F%2Fmonzo.com%2Fsign-up&adj_engagement_type=fallback_click",
            "https://app.adjust.com/1dxkjz95?redirect=https%3A%2F%2Fmonzo.com%2Fsign-up&engagement_type=fallback_click",
            "https://monzo.com/help",
            "https://monzo.com/about",
            "https://monzo.com/us",
            "https://monzo.com/blog",
            "https://monzo.com/press",
            "https://web.monzo.com/",
            "https://monzo.com/investor-information",
            "https://monzo.com/supporting-all-our-customers",
            "https://monzo.com/helping-everyone-belong-at-monzo",
            "https://monzo.com/fraud",
            "https://monzo.com/tone-of-voice",
            "https://monzo.com/business-banking",
            "https://monzo.com/modern-slavery-statements",
            "https://monzo.com/faq",
            "https://monzo.com/legal/terms-and-conditions/",
            "https://monzo.com/legal/fscs-information/",
            "https://monzo.com/legal/privacy-notice/",
            "https://monzo.com/legal/cookie-notice/",
            "https://monzo.com/legal/browser-support-policy/",
            "https://monzo.com/legal/mobile-operating-system-support-policy/",
            "https://monzo.com/information-about-current-account-services",
            "https://monzo.com/service-information",
            "https://twitter.com/monzo",
            "https://www.instagram.com/monzo",
            "https://www.facebook.com/monzobank",
            "https://www.linkedin.com/company/monzo-bank",
            "https://www.youtube.com/monzobank"
        ];
        links
            .iter()
            .map(|input| Url::parse(input).unwrap())
            .collect()
    }

    fn build_robots_txt() -> String {
        r#"User-agent: *
            Disallow: /docs/
            Disallow: /referral/
            Disallow: /-staging-referral/
            Disallow: /install/
            Disallow: /blog/authors/
            Disallow: /-deeplinks/"#
            .to_string()
    }

    #[tokio::test]
    async fn filter_new_urls() {
        let start_url = Url::parse("https://monzo.com/").unwrap();
        let subdomain = start_url.host_str().unwrap().to_string();
        let url_filter = UrlFilter::new(100, subdomain, build_robots_txt());
        let urls = build_urls();
        let filtered = url_filter.filter(urls);
        assert_eq!(filtered.len(), 21);
    }

    #[tokio::test]
    async fn filter_visited_urls() {
        let start_url = Url::parse("https://monzo.com/").unwrap();
        let subdomain = start_url.host_str().unwrap().to_string();
        let url_filter = UrlFilter::new(100, subdomain, build_robots_txt());
        let mut urls = build_urls();
        url_filter.filter(urls.clone());

        // Add a new URL to the dataset and filter again, only this URL should be returned.
        let new_url = Url::parse("https://monzo.com/gonzo").unwrap();
        urls.insert(new_url.clone());
        let filtered = url_filter.filter(urls);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.into_iter().next().unwrap(), new_url);
    }

    #[test]
    fn apply_robots_txt() {
        let start_url = Url::parse("https://monzo.com/").unwrap();
        let subdomain = start_url.host_str().unwrap().to_string();
        let url_filter = UrlFilter::new(100, subdomain, build_robots_txt());

        let allow_1 = start_url.join("faq/").unwrap();
        let allow_2 = Url::parse("https://instagram.com/monzo").unwrap();
        let allow_3 = start_url.join("legal/docs/").unwrap();
        assert!(url_filter.allowed(&allow_1));
        assert!(url_filter.allowed(&allow_2));
        assert!(url_filter.allowed(&allow_3));

        let disallow_1 = start_url.join("docs/").unwrap();
        let disallow_2 = start_url.join("docs/introduction").unwrap();
        let disallow_3 = start_url.join("docs/legal/").unwrap();
        assert!(!url_filter.allowed(&disallow_1));
        assert!(!url_filter.allowed(&disallow_2));
        assert!(!url_filter.allowed(&disallow_3));
    }
}
