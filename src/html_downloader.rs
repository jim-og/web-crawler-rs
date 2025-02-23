use reqwest::{Client, StatusCode};
use url::Url;

#[derive(Default)]
pub struct HtmlDownloader {
    client: Client,
}

pub struct HtmlPage {
    pub _status: StatusCode, // TODO
    pub body: String,
}

impl HtmlDownloader {
    pub async fn fetch(&self, url: Url) -> Result<HtmlPage, reqwest::Error> {
        // TODO could url be borrowed?
        let response = self
            .client
            .get(url.clone())
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .send()
            .await?;
        Ok(HtmlPage {
            _status: response.status(),
            body: response.text().await?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::HtmlDownloader;
    use url::Url;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn build_mock_server(endpoint: &str, response_body: &str) -> MockServer {
        // Start a background HTTP server on a random local port
        let mock_server = MockServer::start().await;

        // When the MockServer receives a GET request on '/hello' it will respond with a 200.
        Mock::given(method("GET"))
            .and(path(endpoint))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        mock_server
    }

    #[tokio::test]
    async fn fetch_ok() {
        let response_body = "Body of mocked response";
        let mock_server = build_mock_server("/hello", response_body).await;
        let url = format!("{}/hello", mock_server.uri());
        let downloader = HtmlDownloader::default();
        let page = downloader.fetch(Url::parse(&url).unwrap()).await.unwrap();

        assert_eq!(page._status.as_u16(), 200);
        assert_eq!(page.body, response_body);
    }

    #[tokio::test]
    async fn fetch_not_found() {
        let response_body = "Body of mocked response";
        let mock_server = build_mock_server("/hello", response_body).await;
        let url = format!("{}/goodbye", mock_server.uri());
        let downloader = HtmlDownloader::default();
        let page = downloader.fetch(Url::parse(&url).unwrap()).await.unwrap();

        assert_eq!(page._status.as_u16(), 404);
        assert_eq!(page.body, "");
    }
}
