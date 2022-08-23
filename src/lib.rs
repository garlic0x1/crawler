pub mod crawler;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::crawler::*;

    #[tokio::test]
    async fn crawl_test() {
        let mut visited = HashSet::new();
        let mut links = HashSet::new();
        let errs = Crawler::builder()
            .add_default_propagators()
            .whitelist("qiwi-button")
            .user_agent("Mozilla/5.0 (X11; Linux x86_64)...")
            .add_handler("*[href]", |args| {
                if let Some(link) = args.element.unwrap().value().attr("href") {
                    links.insert(link.to_string());
                }
            })
            .on_page(|args: &HandlerArgs| {
                let ustr = args.page.url.to_string();
                if ustr.ends_with(".php") {
                    visited.insert(ustr);
                }
            })
            .depth(3)
            .build()
            .unwrap()
            .crawl("http://plugins.svn.wordpress.org/qiwi-button/")
            .await
            .unwrap();

        println!("{:?}", visited);
        println!("{:?}", links);
        println!("{:?}", errs);
        assert_eq!(visited.len(), 18);
        assert_eq!(links.len(), 61);
    }
}
