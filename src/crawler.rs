use std::collections::HashMap;

use anyhow::Result;
use reqwest::*;
use scraper::*;

pub struct Crawler {
    handlers: HashMap<String, Vec<Box<dyn Fn(ElementRef, Url)>>>,
    propagators: HashMap<String, Vec<Box<dyn Fn(&Self, ElementRef, Url, u32)>>>,
    depth: u32,
    client: Client,
}

impl Crawler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            propagators: HashMap::new(),
            depth: 2,
            client: Client::new(),
        }
    }

    pub fn add_handler<F>(mut self, sel: String, closure: F) -> Self
    where
        F: Fn(ElementRef, Url) + 'static,
    {
        let closure: Box<dyn Fn(ElementRef, Url)> = Box::new(closure);
        if let Some(handlers) = self.handlers.get_mut(&sel) {
            handlers.push(closure)
        } else {
            self.handlers.insert(sel, vec![closure]);
        }
        self
    }

    pub fn add_propagator<F>(mut self, sel: String, closure: F) -> Self
    where
        F: Fn(&Self, ElementRef, Url, u32) + 'static,
    {
        let closure: Box<dyn Fn(&Self, ElementRef, Url, u32)> = Box::new(closure);
        if let Some(propagators) = self.propagators.get_mut(&sel) {
            propagators.push(closure)
        } else {
            self.propagators.insert(sel, vec![closure]);
        }
        self
    }

    pub async fn crawl(&self, url: &str) -> Result<()> {
        let uri = Url::parse(url)?;
        self.visit(uri, self.depth).await?;
        Ok(())
    }

    async fn visit(&self, url: Url, depth: u32) -> Result<()> {
        let res = self.client.get(url.clone()).send().await?;
        let text = res.text().await?;
        let doc = Html::parse_document(&text);

        for handlers in self.handlers.iter() {
            if let Ok(sel) = Selector::parse(handlers.0) {
                for handler in handlers.1 {
                    for el in doc.select(&sel) {
                        handler(el, url.clone());
                    }
                }
            } else {
                eprintln!("invalid selector {}", handlers.0);
            }
        }

        if depth > 0 {
            for propagator in self.propagators.iter() {
                if let Ok(sel) = Selector::parse(propagator.0) {
                    for propagator in propagator.1 {
                        for el in doc.select(&sel) {
                            propagator(self, el, url.clone(), depth - 1);
                        }
                    }
                } else {
                    eprintln!("invalid selector {}", propagator.0);
                }
            }
        }

        Ok(())
    }

    pub fn add_default_propagators(mut self) -> Self {
        let href_prop = |crawler: &Self, el: ElementRef, url: Url, depth: u32| {
            if let Some(href) = el.value().attr("href") {
                println!("propagating {href}");

                if depth > 0 {
                    if let Ok(abs_url) = url.join(href) {
                        println!("absolute url: {abs_url}");
                        crawler.visit(abs_url, depth - 1);
                    }
                }

                // TODO
                // need to figure out how to get absolute url
                // TODO
            }
        };

        let defaults = vec![href_prop];

        for prop in defaults {
            self = self.add_propagator("*[href]".into(), prop);
        }

        self
    }
}
