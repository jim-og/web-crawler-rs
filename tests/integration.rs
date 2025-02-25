#[cfg(test)]
mod tests {
    use url::Url;
    use web_crawler_rs::crawler::Crawler;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn build_site(mock_server: &MockServer) {
        // Mock robots.txt response
        Mock::given(method("GET"))
            .and(path("/robots.txt"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"User-agent: *
                        Disallow: /docs/
                        Disallow: /referral/
                        Disallow: /-staging-referral/
                        Disallow: /install/
                        Disallow: /blog/authors/
                        Disallow: /-deeplinks/"#,
            ))
            .mount(&mock_server)
            .await;

        // Mock the landing page 200 response with links
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                r#"
                    <html>
                        <body>
                            <a href="{0}/a">a</a>
                            <a href="{0}/b">b</a>
                            <a href="{0}/c">c</a>
                            <a href="{0}/docs">docs</a>
                        </body>
                    </html>
                "#,
                mock_server.uri()
            )))
            .mount(&mock_server)
            .await;

        // Mock /a 200 response with links
        Mock::given(method("GET"))
            .and(path("/a"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                r#"
                    <html>
                        <body>
                            <a href="{0}/b">b</a>
                            <a href="{0}/e">e</a>
                            <a href="{0}/f">f</a>
                            <a href="{0}/referral">referral</a>
                        </body>
                    </html>
                "#,
                mock_server.uri()
            )))
            .mount(&mock_server)
            .await;

        // Mock /b 200 response with no links
        Mock::given(method("GET"))
            .and(path("/b"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"
                    <html>
                        <body>
                            Exciting content
                        </body>
                    </html>
                "#,
            ))
            .mount(&mock_server)
            .await;

        // Mock /f 404 response
        Mock::given(method("GET"))
            .and(path("/f"))
            .respond_with(ResponseTemplate::new(404).set_body_string(
                r#"
                    <html>
                        <body>
                            Not found
                        </body>
                    </html>
                "#,
            ))
            .mount(&mock_server)
            .await;
    }

    #[tokio::test]
    async fn end_to_end() {
        // Start a background HTTP server on a random local port
        let mock_server = MockServer::start().await;

        build_site(&mock_server).await;

        let seed = Url::parse(&mock_server.uri()).unwrap();
        let result = Crawler::run(seed).await;

        assert!(result.is_ok());
    }
}
