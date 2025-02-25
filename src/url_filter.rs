use crate::store::Store;
use robotstxt::DefaultMatcher;
use std::collections::HashSet;
use url::Url;

pub struct UrlFilter {
    subdomain: String,
    url_store: Store<Url>,
    robots_txt: String,
}

impl UrlFilter {
    pub fn new(subdomain: String, robots_txt: String) -> Self {
        UrlFilter {
            subdomain,
            url_store: Store::new(),
            robots_txt,
        }
    }

    /// Filter a set of URLs based on the following criteria
    /// 1. Be in the same subdomain.
    /// 2. Are allowed by robots.txt.
    /// 3. Have not been visited before.
    pub fn filter(&self, urls: HashSet<Url>) -> HashSet<Url> {
        let filtered: Vec<Url> = urls
            .into_iter()
            // Exclude URLs which do not match the subdomain.
            .filter(|url| url.host_str().unwrap_or("") == self.subdomain)
            // Exclude URLs which are not allowed by robots.txt
            .filter(|url| self.allowed(url))
            .collect();

        filtered
            .into_iter()
            // Exclude URLs which have been seen before, add new URLs to data store.
            .filter_map(|url| {
                if self.url_store.insert(url.clone()) {
                    Some(url) // New, keep
                } else {
                    None // Already seen, skip
                }
            })
            .collect()
    }

    /// Determine whether the URL is allowed by robots.txt
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
            "https://we83.adj.st/home?adj_t=1dj2rkno_1dxkjz95&adj_redirect=https%3A%2F%2Fexample.com%2Fsign-up&adj_engagement_type=fallback_click",
            "https://uk.trustpilot.com/review/www.example.com",
            "https://we83.adj.st/home?adj_t=1dj2rkno&adj_fallback=https%3A%2F%2Fexample.com%2Fdownload&adj_engagement_type=fallback_click",
            "https://app.adjust.com/1dxkjz95?fallback=https%3A%2F%2Fexample.com%2Fdownload&engagement_type=fallback_click",
            "https://www.psr.org.uk/app-fraud-data",
            "https://we83.adj.st/home?adj_t=1dj2rkno&adj_redirect=https%3A%2F%2Fexample.com%2Fsign-up&adj_engagement_type=fallback_click",
            "https://app.adjust.com/1dxkjz95?redirect=https%3A%2F%2Fexample.com%2Fsign-up&engagement_type=fallback_click",
            "https://example.com/help",
            "https://example.com/about",
            "https://example.com/us",
            "https://example.com/blog",
            "https://example.com/press",
            "https://web.example.com/",
            "https://example.com/investor-information",
            "https://example.com/supporting-all-our-customers",
            "https://example.com/helping-everyone-belong-at-example",
            "https://example.com/fraud",
            "https://example.com/tone-of-voice",
            "https://example.com/business-banking",
            "https://example.com/modern-slavery-statements",
            "https://example.com/faq",
            "https://example.com/legal/terms-and-conditions/",
            "https://example.com/legal/fscs-information/",
            "https://example.com/legal/privacy-notice/",
            "https://example.com/legal/cookie-notice/",
            "https://example.com/legal/browser-support-policy/",
            "https://example.com/legal/mobile-operating-system-support-policy/",
            "https://example.com/information-about-current-account-services",
            "https://example.com/service-information",
            "https://twitter.com/example",
            "https://www.instagram.com/example",
            "https://www.facebook.com/example",
            "https://www.linkedin.com/company/example",
            "https://www.youtube.com/example"
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
        let start_url = Url::parse("https://example.com/").unwrap();
        let subdomain = start_url.host_str().unwrap().to_string();
        let url_filter = UrlFilter::new(subdomain, build_robots_txt());
        let urls = build_urls();
        let filtered = url_filter.filter(urls);
        assert_eq!(filtered.len(), 21);
    }

    #[tokio::test]
    async fn filter_visited_urls() {
        let start_url = Url::parse("https://example.com/").unwrap();
        let subdomain = start_url.host_str().unwrap().to_string();
        let url_filter = UrlFilter::new(subdomain, build_robots_txt());
        let mut urls = build_urls();
        url_filter.filter(urls.clone());

        // Add a new URL to the dataset and filter again, only this URL should be returned.
        let new_url = Url::parse("https://example.com/gonzo").unwrap();
        urls.insert(new_url.clone());
        let filtered = url_filter.filter(urls);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.into_iter().next().unwrap(), new_url);
    }

    #[test]
    fn apply_robots_txt() {
        let start_url = Url::parse("https://example.com/").unwrap();
        let subdomain = start_url.host_str().unwrap().to_string();
        let url_filter = UrlFilter::new(subdomain, build_robots_txt());

        let allow_1 = start_url.join("faq/").unwrap();
        let allow_2 = Url::parse("https://instagram.com/example").unwrap();
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
