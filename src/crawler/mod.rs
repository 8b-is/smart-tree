//! V3R1T4S-compatible Web Crawler & Spider for Smart Tree
//!
//! Provides a polite, rate-limited web crawler with:
//! - robots.txt compliance
//! - Rate limiting with configurable delays
//! - Content extraction (text, links, metadata)
//! - Integration with Smart Tree's MCP tools
//! - V3R1T4S-compatible output format

use anyhow::{Context, Result};
use reqwest::{Client, header::{USER_AGENT, HeaderMap, HeaderValue}};
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use url::Url;

/// Configuration for the crawler
#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    /// User agent string
    pub user_agent: String,
    /// Maximum concurrent requests
    pub max_concurrent: usize,
    /// Delay between requests to same domain
    pub request_delay: Duration,
    /// Maximum depth to crawl
    pub max_depth: usize,
    /// Maximum pages to crawl
    pub max_pages: usize,
    /// Follow redirects
    pub follow_redirects: bool,
    /// Respect robots.txt
    pub respect_robots_txt: bool,
    /// Custom headers
    pub custom_headers: HeaderMap,
    /// Timeout for requests
    pub timeout: Duration,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("SmartTree-V3R1T4S-Crawler/1.0 (compatible; +https://github.com/8b-is/smart-tree)"),
        );
        Self {
            user_agent: "SmartTree-V3R1T4S-Crawler/1.0".to_string(),
            max_concurrent: 4,
            request_delay: Duration::from_millis(500),
            max_depth: 3,
            max_pages: 100,
            follow_redirects: true,
            respect_robots_txt: true,
            custom_headers: headers,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Robots.txt parser cache entry
#[derive(Debug, Clone)]
struct RobotsEntry {
    disallow: Vec<String>,
    allow: Vec<String>,
    crawl_delay: Option<Duration>,
    expires: Instant,
}

/// Crawled page result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawledPage {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<String>,
    pub metadata: PageMetadata,
    pub depth: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub content_hash: String,
}

/// Page metadata extracted from HTML
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PageMetadata {
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub author: Option<String>,
    pub published_time: Option<String>,
    pub modified_time: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<String>,
    pub canonical_url: Option<String>,
    pub language: Option<String>,
    pub charset: Option<String>,
    pub viewport: Option<String>,
    pub generator: Option<String>,
}

/// Crawler statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CrawlerStats {
    pub pages_crawled: usize,
    pub pages_failed: usize,
    pub bytes_downloaded: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub domains_visited: HashSet<String>,
    pub errors: Vec<CrawlError>,
}

/// Crawl error
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawlError {
    pub url: String,
    pub error: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Main crawler struct
pub struct WebCrawler {
    client: Client,
    config: CrawlerConfig,
    semaphore: Arc<Semaphore>,
    visited: Arc<tokio::sync::Mutex<HashSet<String>>>,
    queue: Arc<tokio::sync::Mutex<VecDeque<(String, usize)>>>, // (url, depth)
    robots_cache: Arc<tokio::sync::Mutex<std::collections::HashMap<String, RobotsEntry>>>,
    stats: Arc<tokio::sync::Mutex<CrawlerStats>>,
    results: Arc<tokio::sync::Mutex<Vec<CrawledPage>>>,
}

impl WebCrawler {
    /// Create a new web crawler with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(CrawlerConfig::default())
    }

    /// Create a new web crawler with custom configuration
    pub fn with_config(config: CrawlerConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .redirect(if config.follow_redirects {
                reqwest::redirect::Policy::limited(10)
            } else {
                reqwest::redirect::Policy::none()
            })
            .user_agent(&config.user_agent)
            .default_headers(config.custom_headers.clone())
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            config,
            semaphore: Arc::new(Semaphore::new(4)),
            visited: Arc::new(tokio::sync::Mutex::new(HashSet::new())),
            queue: Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
            robots_cache: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            stats: Arc::new(tokio::sync::Mutex::new(CrawlerStats::default())),
            results: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        })
    }

    /// Start crawling from a seed URL
    pub async fn crawl(&self, seed_url: &str) -> Result<Vec<CrawledPage>> {
        let seed = Url::parse(seed_url).context("Invalid seed URL")?;
        let domain = seed.domain().unwrap_or("").to_string();

        // Initialize stats
        {
            let mut stats = self.stats.lock().await;
            stats.start_time = chrono::Utc::now();
            stats.domains_visited.insert(domain.clone());
        }

        // Add seed to queue
        {
            let mut queue = self.queue.lock().await;
            queue.push_back((seed_url.to_string(), 0));
        }

        // Process queue
        self.process_queue().await?;

        // Finalize stats
        {
            let mut stats = self.stats.lock().await;
            stats.end_time = Some(chrono::Utc::now());
        }

        let results = self.results.lock().await.clone();
        Ok(results)
    }

    /// Process the crawl queue
    async fn process_queue(&self) -> Result<()> {
        let mut handles = Vec::new();

        // Spawn worker tasks
        for _ in 0..self.config.max_concurrent {
            let crawler = self.clone();
            handles.push(tokio::spawn(async move {
                crawler.worker().await;
            }));
        }

        // Wait for all workers
        for handle in handles {
            handle.await?;
        }

        Ok(())
    }

    /// Worker task that processes URLs from the queue
    async fn worker(&self) {
        loop {
            let (url, depth) = {
                let mut queue = self.queue.lock().await;
                queue.pop_front()
            };

            let Some((url, depth)) = url else {
                break; // Queue empty
            };

            if depth > self.config.max_depth {
                continue;
            }

            // Check if already visited
            let already_visited = {
                let visited = self.visited.lock().await;
                visited.contains(&url)
            };
            if already_visited {
                continue;
            }

            // Check page limit
            let at_limit = {
                let stats = self.stats.lock().await;
                stats.pages_crawled >= self.config.max_pages
            };
            if at_limit {
                continue;
            }

            // Mark as visited
            {
                let mut visited = self.visited.lock().await;
                visited.insert(url.clone());
            }

            // Check robots.txt
            if self.config.respect_robots_txt {
                if !self.check_robots_txt(&url).await {
                    continue;
                }
            }

            // Rate limiting
            self.rate_limit(&url).await;

            // Fetch and parse
            match self.fetch_and_parse(&url, depth).await {
                Ok(page) => {
                    // Add to results
                    {
                        let mut results = self.results.lock().await;
                        results.push(page.clone());
                    }

                    // Update stats
                    {
                        let mut stats = self.stats.lock().await;
                        stats.pages_crawled += 1;
                        stats.bytes_downloaded += page.content.len() as u64;
                    }

                    // Extract links and add to queue
                    if depth < self.config.max_depth {
                        self.enqueue_links(&page, depth + 1).await;
                    }
                }
                Err(e) => {
                    // Record error
                    let mut stats = self.stats.lock().await;
                    stats.pages_failed += 1;
                    stats.errors.push(CrawlError {
                        url: url.clone(),
                        error: e.to_string(),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }
    }

    /// Check robots.txt for a URL
    async fn check_robots_txt(&self, url: &str) -> bool {
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return true,
        };

        let domain = parsed.domain().unwrap_or("").to_string();
        let path = parsed.path();

        // Check cache
        {
            let cache = self.robots_cache.lock().await;
            if let Some(entry) = cache.get(&domain) {
                if Instant::now() < entry.expires {
                    return self.path_allowed(path, &entry.disallow, &entry.allow);
                }
            }
        }

        // Fetch robots.txt
        let robots_url = format!("{}://{}/robots.txt", parsed.scheme(), domain);
        let robots_txt = match self.client.get(&robots_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                resp.text().await.unwrap_or_default()
            }
            _ => String::new(),
        };

        // Parse robots.txt
        let entry = self.parse_robots_txt(&robots_txt);

        // Cache it
        {
            let mut cache = self.robots_cache.lock().await;
            cache.insert(domain, entry.clone());
        }

        self.path_allowed(path, &entry.disallow, &entry.allow)
    }

    /// Parse robots.txt content
    fn parse_robots_txt(&self, content: &str) -> RobotsEntry {
        let mut disallow = Vec::new();
        let mut allow = Vec::new();
        let mut crawl_delay = None;
        let mut current_user_agent = String::new();
        let mut applies = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                match key.as_str() {
                    "user-agent" => {
                        current_user_agent = value.to_string();
                        applies = value == "*" || value.to_lowercase().contains("smarttree") || value.to_lowercase().contains("bot");
                    }
                    "disallow" => {
                        if applies && !value.is_empty() {
                            disallow.push(value.to_string());
                        }
                    }
                    "allow" => {
                        if applies && !value.is_empty() {
                            allow.push(value.to_string());
                        }
                    }
                    "crawl-delay" => {
                        if applies {
                            if let Ok(delay) = value.parse::<u64>() {
                                crawl_delay = Some(Duration::from_secs(delay));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        RobotsEntry {
            disallow,
            allow,
            crawl_delay,
            expires: Instant::now() + Duration::from_secs(3600), // 1 hour cache
        }
    }

    /// Check if a path is allowed by robots.txt rules
    fn path_allowed(&self, path: &str, disallow: &[String], allow: &[String]) -> bool {
        // Check allow rules first (more specific)
        for rule in allow {
            if path.starts_with(rule) {
                return true;
            }
        }

        // Check disallow rules
        for rule in disallow {
            if path.starts_with(rule) {
                return false;
            }
        }

        true
    }

    /// Rate limiting per domain
    async fn rate_limit(&self, url: &str) {
        let delay = self.config.request_delay;
        tokio::time::sleep(delay).await;
    }

    /// Fetch and parse a single URL
    async fn fetch_and_parse(&self, url: &str, depth: usize) -> Result<CrawledPage> {
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("HTTP {}", response.status());
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/html") {
            anyhow::bail!("Non-HTML content type: {}", content_type);
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Extract title
        let title_selector = Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty());

        // Extract metadata
        let metadata = self.extract_metadata(&document);

        // Extract main content
        let content = self.extract_content(&document);

        // Extract links
        let links = self.extract_links(&document, url)?;

        // Compute content hash
        let content_hash = format!("{:x}", md5::compute(&content));

        Ok(CrawledPage {
            url: url.to_string(),
            title,
            content,
            links,
            metadata,
            depth,
            timestamp: chrono::Utc::now(),
            content_hash,
        })
    }

    /// Extract page metadata from HTML
    fn extract_metadata(&self, document: &Html) -> PageMetadata {
        let mut metadata = PageMetadata::default();

        // Meta tags
        let meta_selector = Selector::parse("meta").unwrap();
        for el in document.select(&meta_selector) {
            if let Some(name) = el.value().attr("name") {
                if let Some(content) = el.value().attr("content") {
                    match name.to_lowercase().as_str() {
                        "description" => metadata.description = Some(content.to_string()),
                        "keywords" => metadata.keywords = Some(content.to_string()),
                        "author" => metadata.author = Some(content.to_string()),
                        "viewport" => metadata.viewport = Some(content.to_string()),
                        "generator" => metadata.generator = Some(content.to_string()),
                        _ => {}
                    }
                }
            }

            if let Some(property) = el.value().attr("property") {
                if let Some(content) = el.value().attr("content") {
                    match property.as_str() {
                        "og:title" => metadata.og_title = Some(content.to_string()),
                        "og:description" => metadata.og_description = Some(content.to_string()),
                        "og:image" => metadata.og_image = Some(content.to_string()),
                        _ => {}
                    }
                }
            }
        }

        // Canonical URL
        let canonical_selector = Selector::parse("link[rel=canonical]").unwrap();
        if let Some(el) = document.select(&canonical_selector).next() {
            metadata.canonical_url = el.value().attr("href").map(|s| s.to_string());
        }

        // Language
        let html_selector = Selector::parse("html").unwrap();
        if let Some(el) = document.select(&html_selector).next() {
            metadata.language = el.value().attr("lang").map(|s| s.to_string());
        }

        // Charset
        let charset_selector = Selector::parse("meta[charset]").unwrap();
        if let Some(el) = document.select(&charset_selector).next() {
            metadata.charset = el.value().attr("charset").map(|s| s.to_string());
        }

        // Published/modified times
        let time_selector = Selector::parse("time[datetime]").unwrap();
        for el in document.select(&time_selector) {
            if let Some(datetime) = el.value().attr("datetime") {
                if metadata.published_time.is_none() {
                    metadata.published_time = Some(datetime.to_string());
                } else {
                    metadata.modified_time = Some(datetime.to_string());
                }
            }
        }

        metadata
    }

    /// Extract main content from HTML
    fn extract_content(&self, document: &Html) -> String {
        // Try common content selectors in order of preference
        let content_selectors = [
            "main",
            "article",
            "[role=main]",
            ".content",
            ".post",
            ".entry",
            "#content",
            "#main",
        ];

        for selector_str in &content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(el) = document.select(selector).next() {
                    let text = el.text().collect::<Vec<_>>().join(" ");
                    if text.len() > 100 {
                        return text;
                    }
                }
            }
        }

        // Fallback: body text
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(el) = document.select(&body_selector).next() {
                return el.text().collect::<Vec<_>>().join(" ");
            }
        }

        String::new()
    }

    /// Extract links from HTML
    fn extract_links(&self, document: &Html, base_url: &str) -> Result<Vec<String>> {
        let link_selector = Selector::parse("a[href]").unwrap();
        let base = Url::parse(base_url)?;
        let mut links = Vec::new();

        for el in document.select(&link_selector) {
            if let Some(href) = el.value().attr("href") {
                if let Ok(url) = base.join(href) {
                    let url_str = url.to_string();
                    // Only include HTTP/HTTPS links
                    if url_str.starts_with("http://") || url_str.starts_with("https://") {
                        links.push(url_str);
                    }
                }
            }
        }

        // Deduplicate
        links.sort();
        links.dedup();

        Ok(links)
    }

    /// Enqueue extracted links
    async fn enqueue_links(&self, page: &CrawledPage, depth: usize) {
        let mut queue = self.queue.lock().await;
        for link in &page.links {
            if !queue.iter().any(|(u, _)| u == link) {
                queue.push_back((link.clone(), depth));
            }
        }
    }

    /// Get crawler statistics
    pub async fn stats(&self) -> CrawlerStats {
        self.stats.lock().await.clone()
    }

    /// Get crawled results
    pub async fn results(&self) -> Vec<CrawledPage> {
        self.results.lock().await.clone()
    }

    /// Export results to JSON
    pub async fn export_json(&self) -> Result<String> {
        let results = self.results.lock().await.clone();
        serde_json::to_string_pretty(&results).context("Failed to serialize results")
    }

    /// Export results in V3R1T4S-compatible format
    pub async fn export_v3r1t4s(&self) -> Result<String> {
        let results = self.results.lock().await.clone();
        let stats = self.stats.lock().await.clone();

        let export = serde_json::json!({
            "v3r1t4s_crawl": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "stats": {
                    "pages_crawled": stats.pages_crawled,
                    "pages_failed": stats.pages_failed,
                    "bytes_downloaded": stats.bytes_downloaded,
                    "domains_visited": stats.domains_visited.len(),
                    "errors": stats.errors.len(),
                },
                "pages": results.iter().map(|p| {
                    serde_json::json!({
                        "url": p.url,
                        "title": p.title,
                        "depth": p.depth,
                        "content_length": p.content.len(),
                        "content_hash": p.content_hash,
                        "links_count": p.links.len(),
                        "timestamp": p.timestamp.to_rfc3339(),
                    })
                }).collect::<Vec<_>>(),
            }
        });

        serde_json::to_string_pretty(&export).context("Failed to serialize V3R1T4S export")
    }
}

impl Clone for WebCrawler {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            semaphore: self.semaphore.clone(),
            visited: self.visited.clone(),
            queue: self.queue.clone(),
            robots_cache: self.robots_cache.clone(),
            stats: self.stats.clone(),
            results: self.results.clone(),
        }
    }
}

/// Convenience function for quick crawling
pub async fn quick_crawl(url: &str, max_pages: usize) -> Result<Vec<CrawledPage>> {
    let mut config = CrawlerConfig::default();
    config.max_pages = max_pages;
    config.max_depth = 2;

    let crawler = WebCrawler::with_config(config)?;
    crawler.crawl(url).await
}

/// Crawl and export in V3R1T4S format
pub async fn crawl_v3r1t4s(url: &str, max_pages: usize) -> Result<String> {
    let mut config = CrawlerConfig::default();
    config.max_pages = max_pages;
    config.max_depth = 2;

    let crawler = WebCrawler::with_config(config)?;
    crawler.crawl(url).await?;
    crawler.export_v3r1t4s().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawler_config_default() {
        let config = CrawlerConfig::default();
        assert_eq!(config.max_concurrent, 4);
        assert_eq!(config.max_depth, 3);
        assert_eq!(config.max_pages, 100);
    }

    #[tokio::test]
    async fn test_robots_txt_parsing() {
        let crawler = WebCrawler::new().unwrap();
        let robots_txt = r#"
User-agent: *
Disallow: /private/
Disallow: /admin/
Allow: /admin/public/
Crawl-delay: 10
"#;
        let entry = crawler.parse_robots_txt(robots_txt);
        
        assert_eq!(entry.disallow, vec!["/private/", "/admin/"]);
        assert_eq!(entry.allow, vec!["/admin/public/"]);
        assert_eq!(entry.crawl_delay, Some(Duration::from_secs(10)));
    }

    #[tokio::test]
    async fn test_path_allowed() {
        let crawler = WebCrawler::new().unwrap();
        let disallow = vec!["/private/".to_string(), "/admin/".to_string()];
        let allow = vec!["/admin/public/".to_string()];

        assert!(crawler.path_allowed("/public/page", &disallow, &allow));
        assert!(!crawler.path_allowed("/private/secret", &disallow, &allow));
        assert!(!crawler.path_allowed("/admin/dashboard", &disallow, &allow));
        assert!(crawler.path_allowed("/admin/public/info", &disallow, &allow));
    }
}