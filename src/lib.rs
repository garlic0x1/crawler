pub mod crawler;

#[cfg(test)]
mod tests {
    use super::crawler::*;
    use reqwest::Url;
    use scraper::ElementRef;
    use std::collections::HashSet;

    // this does not work with tokio test?
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn it_works() {
        let mut seen: HashSet<String> = HashSet::new();
        Crawler::builder()
            .add_default_propagators()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64)...".into())
            .add_handler("*[href]", |el: ElementRef, url: Url| {
                if let Some(href) = el.value().attr("href") {
                    if let Ok(abs_url) = url.join(href) {
                        seen.insert(abs_url.to_string());
                    } else {
                        seen.insert(href.to_string());
                    }
                }
            })
            .depth(1)
            .build()
            .unwrap()
            .crawl("https://vim.org/weird.php")
            .await
            .unwrap();

        assert_eq!(seen.len(), 32);
    }
}
