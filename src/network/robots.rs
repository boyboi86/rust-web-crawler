use anyhow::Error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{Instant, sleep};
use url::Url;

use crate::config::defaults;
use crate::core::RobotsChecker;

// Type alias for complex robots cache entry
type RobotsCacheEntry = (String, Option<u64>, Instant);

/// Robots.txt handler with caching
pub struct RobotsCache {
    cache: Arc<Mutex<HashMap<String, RobotsCacheEntry>>>,
}

impl Default for RobotsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl RobotsCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_cache(&self) -> Arc<Mutex<HashMap<String, RobotsCacheEntry>>> {
        self.cache.clone()
    }
}

/// Robots.txt checker implementation
pub struct RobotsHandler {
    cache: RobotsCache,
    client: reqwest::Client,
}

impl RobotsHandler {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            cache: RobotsCache::new(),
            client,
        }
    }

    pub async fn get_robots_cache(
        &self,
    ) -> Arc<Mutex<HashMap<String, (String, Option<u64>, Instant)>>> {
        self.cache.get_cache().await
    }
}

impl RobotsChecker for RobotsHandler {
    /// Check if URL is allowed by robots.txt
    async fn is_allowed_by_robots(&self, url: &Url) -> Result<bool, Error> {
        let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));
        let robots_url = format!("{}/robots.txt", base_url);

        // Check cache first with TTL validation
        {
            let mut cache = self.cache.cache.lock().await;
            if let Some((robots_content, crawl_delay, cached_at)) = cache.get(&base_url) {
                // Check if cache entry is still valid (24 hours TTL)
                if cached_at.elapsed()
                    < Duration::from_secs(defaults::ROBOTS_CACHE_TTL_HOURS * 3600)
                {
                    // Apply crawl delay if specified in robots.txt
                    if let Some(delay) = crawl_delay {
                        sleep(Duration::from_millis(*delay)).await;
                    }
                    return Ok(self.parse_robots_txt(robots_content, url.path()).0);
                } else {
                    // Remove expired entry
                    cache.remove(&base_url);
                }
            }
        }

        // Fetch and parse robots.txt
        match self.client.get(&robots_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let robots_content = response.text().await?;
                    let (is_allowed, crawl_delay) =
                        self.parse_robots_txt(&robots_content, url.path());

                    // Cache the result with crawl delay and timestamp
                    {
                        let mut cache = self.cache.cache.lock().await;
                        cache.insert(base_url, (robots_content, crawl_delay, Instant::now()));
                    }

                    // Apply crawl delay if specified
                    if let Some(delay) = crawl_delay {
                        sleep(Duration::from_millis(delay)).await;
                    }

                    Ok(is_allowed)
                } else {
                    // If robots.txt is not found, assume allowed
                    Ok(true)
                }
            }
            Err(_) => {
                // If we can't fetch robots.txt, assume allowed
                Ok(true)
            }
        }
    }

    /// Enhanced robots.txt parser - handles User-agent, Allow, Disallow, and Crawl-delay
    fn parse_robots_txt(&self, robots_content: &str, path: &str) -> (bool, Option<u64>) {
        let mut in_wildcard_section = false;
        let mut is_allowed = true;
        let mut crawl_delay: Option<u64> = None;

        for line in robots_content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for User-agent directive
            if line.to_lowercase().starts_with("user-agent:") {
                let user_agent = line.split(':').nth(1).unwrap_or("").trim();
                in_wildcard_section = user_agent == "*";
                continue;
            }

            // Only process rules for wildcard user agent
            if in_wildcard_section {
                if line.to_lowercase().starts_with("disallow:") {
                    let disallowed_path = line.split(':').nth(1).unwrap_or("").trim();
                    if !disallowed_path.is_empty() && path.starts_with(disallowed_path) {
                        is_allowed = false;
                    }
                } else if line.to_lowercase().starts_with("allow:") {
                    let allowed_path = line.split(':').nth(1).unwrap_or("").trim();
                    if !allowed_path.is_empty() && path.starts_with(allowed_path) {
                        is_allowed = true;
                    }
                } else if line.to_lowercase().starts_with("crawl-delay:") {
                    // Parse crawl delay (in seconds, convert to milliseconds)
                    if let Some(delay_str) = line.split(':').nth(1)
                        && let Ok(delay_seconds) = delay_str.trim().parse::<f64>()
                    {
                        crawl_delay =
                            Some((delay_seconds * defaults::SECONDS_TO_MS_MULTIPLIER) as u64);
                    }
                } else if line.to_lowercase().starts_with("request-rate:") {
                    // Parse request rate (requests per time period)
                    // Format: "Request-rate: requests/seconds"
                    if let Some(rate_str) = line.split(':').nth(1) {
                        let rate_parts: Vec<&str> = rate_str.trim().split('/').collect();
                        if rate_parts.len() == 2
                            && let (Ok(requests), Ok(seconds)) =
                                (rate_parts[0].parse::<f64>(), rate_parts[1].parse::<f64>())
                        {
                            // Convert to delay between requests in milliseconds
                            let delay_between_requests =
                                (seconds / requests) * defaults::SECONDS_TO_MS_MULTIPLIER;
                            crawl_delay = Some(delay_between_requests as u64);
                        }
                    }
                }
            }
        }

        (is_allowed, crawl_delay)
    }
}
