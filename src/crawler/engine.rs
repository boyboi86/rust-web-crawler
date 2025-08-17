use anyhow::Error;
use bloom::{ASMS, BloomFilter};
use futures::stream::{self, StreamExt};
use rand::Rng;
use reqwest::{Client, Proxy, redirect::Policy};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use url::Url;

use crate::config::{WebCrawlerConfig, defaults};
use crate::core::{ContentProcessor, DnsResolver, HttpClientManager, LangType, RobotsChecker};
use crate::logging::CrawlEventLogger;
use crate::network::{DnsCache, GlobalRateLimiter, RobotsHandler};
use crate::processing::ContentExtractor;

/// Enhanced web crawler with trait implementations
pub struct WebCrawler {
    client: Client,
    visited_urls_bloom: Arc<Mutex<BloomFilter>>,
    semaphore: Arc<Semaphore>,
    min_word_length: usize,
    accepted_languages: Vec<LangType>,
    proxy_pool: Vec<String>,
    delay_ms: u64,
    rate_limiter: Arc<GlobalRateLimiter>,
    dns_resolver: DnsCache,
    robots_handler: RobotsHandler,
    content_processor: ContentExtractor,
    proxy_clients: Arc<Mutex<HashMap<String, Client>>>,
    event_logger: CrawlEventLogger,
}

impl WebCrawler {
    pub fn new(
        config: WebCrawlerConfig,
        max_concurrent_requests: usize,
        _max_depth: usize,
    ) -> Result<Self, Error> {
        Self::new_with_session(config, max_concurrent_requests, _max_depth, None)
    }

    /// Create a new WebCrawler with an optional session ID
    pub fn new_with_session(
        config: WebCrawlerConfig,
        max_concurrent_requests: usize,
        _max_depth: usize,
        session_id: Option<String>,
    ) -> Result<Self, Error> {
        let client = Client::builder()
            .redirect(Policy::limited(defaults::MAX_REDIRECTS))
            .user_agent(config.user_agent.clone())
            .timeout(Duration::from_secs(defaults::REQUEST_TIMEOUT_SECS))
            .build()?;

        // Initialize Bloom filter with capacity for 1M URLs and 1% false positive rate
        let visited_urls_bloom = Arc::new(Mutex::new(BloomFilter::with_rate(
            defaults::BLOOM_FALSE_POSITIVE_RATE,
            defaults::BLOOM_CAPACITY,
        )));
        let semaphore = Arc::new(Semaphore::new(max_concurrent_requests));

        // Initialize rate limiter with configured limits
        let default_rate_limit = config.default_rate_limit.unwrap_or_default();
        let mut rate_limiter = GlobalRateLimiter::new(default_rate_limit);

        // Add domain-specific rate limits if configured
        if let Some(domain_limits) = config.domain_rate_limits {
            for (domain, limit) in domain_limits {
                rate_limiter.add_domain_limit(domain, limit);
            }
        }

        // Initialize components
        let dns_resolver = DnsCache::new();
        let robots_handler = RobotsHandler::new(client.clone());
        let content_processor = ContentExtractor::new(
            config.accepted_languages.clone(),
            config.latin_word_filter.clone(),
        )?;

        // Create session ID and event logger
        let session_id = session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let event_logger = CrawlEventLogger::new(session_id.clone());

        // Initialize logging if configured
        if let Some(_logging_config) = &config.logging_config {
            // Simple console logging initialization
            if let Err(e) = tracing_subscriber::fmt::try_init() {
                tracing::warn!(
                    error = %e,
                    session_id = %session_id,
                    "Logging already initialized or failed to initialize"
                );
            }
        }

        Ok(Self {
            client,
            visited_urls_bloom,
            semaphore,
            min_word_length: config.min_word_length,
            accepted_languages: config.accepted_languages,
            proxy_pool: config.proxy_pool,
            delay_ms: defaults::DEFAULT_POLITENESS_DELAY_MS,
            rate_limiter: Arc::new(rate_limiter),
            dns_resolver,
            robots_handler,
            content_processor,
            proxy_clients: Arc::new(Mutex::new(HashMap::new())),
            event_logger,
        })
    }

    /// Main crawling method
    pub async fn init_crawling(&self, url: Url) -> Result<Option<String>, Error> {
        let start_time = Instant::now();

        // Log crawl start
        self.event_logger.log_crawl_start(
            &url,
            None,
            Some(crate::config::defaults::DEFAULT_WEBCRAWLER_USER_AGENT),
        );

        // 1. Check if URL already visited using Bloom filter
        let url_str = url.as_str();
        {
            let mut bloom = self.visited_urls_bloom.lock().await;
            if bloom.contains(&url_str.to_string()) {
                self.event_logger.log_crawl_failure(
                    &url,
                    start_time.elapsed(),
                    "URL already visited (Bloom filter)",
                    None,
                    None,
                    false,
                );
                return Ok(None); // Probably already visited
            }
            bloom.insert(&url_str.to_string());
        }

        // 2. Check robots.txt compliance
        if !self.robots_handler.is_allowed_by_robots(&url).await? {
            let robots_url = format!(
                "{}://{}/robots.txt",
                url.scheme(),
                url.host_str().unwrap_or("unknown")
            );
            self.event_logger.log_robots_blocked(&url, &robots_url);
            return Ok(None);
        }

        // 3. Apply domain-specific rate limiting (BEFORE acquiring semaphore)
        let domain = url.host_str().unwrap_or("unknown").to_string();
        let rate_limit_start = Instant::now();
        self.rate_limiter.check_and_wait(&domain).await?;
        let rate_limit_duration = rate_limit_start.elapsed();

        if rate_limit_duration.as_millis()
            > crate::config::defaults::RATE_LIMIT_LOG_THRESHOLD_MS as u128
        {
            self.event_logger.log_rate_limited(
                &url,
                rate_limit_duration.as_millis() as u64,
                &domain,
            );
        }

        // 4. Acquire semaphore permit (concurrency control)
        let _permit = self.semaphore.acquire().await?;

        // 5. Add politeness delay (reduced since rate limiting handles most timing)
        sleep(Duration::from_millis(
            self.delay_ms / defaults::POLITENESS_DELAY_DIVISOR,
        ))
        .await;

        // 6. Pre-resolve DNS to warm up cache
        if let Some(host) = url.host_str() {
            // This will cache the DNS resolution for future requests
            let _ = self.dns_resolver.resolve_domain(host).await;
        }

        // 7. Create client with random proxy if available
        let client = self.create_client_with_proxy().await?;

        // 8. Fetch with randomized headers
        let user_agent = self.get_random_user_agent();
        let proxy_info = if !self.proxy_pool.is_empty() {
            Some("proxy") // Would need to track which proxy was actually used
        } else {
            None
        };

        let response_result = client
            .get(url.clone())
            .header("User-Agent", user_agent)
            .header("Accept", defaults::ACCEPT_HEADER)
            .header("Accept-Language", &self.get_accept_language_header())
            .header("Accept-Encoding", defaults::ACCEPT_ENCODING_HEADER)
            .header("Connection", defaults::CONNECTION_HEADER)
            .header(
                "Upgrade-Insecure-Requests",
                defaults::UPGRADE_INSECURE_REQUESTS,
            )
            .send()
            .await;

        let response = match response_result {
            Ok(resp) => resp,
            Err(e) => {
                self.event_logger.log_crawl_failure(
                    &url,
                    start_time.elapsed(),
                    &format!("Network error: {}", e),
                    None,
                    None,
                    false,
                );
                return Err(e.into());
            }
        };

        // Check HTTP status code
        let status = response.status();
        let status_code = status.as_u16();
        if !status.is_success() {
            let error_msg = format!(
                "HTTP error: {} {}",
                status_code,
                status.canonical_reason().unwrap_or("Unknown")
            );

            self.event_logger.log_crawl_failure(
                &url,
                start_time.elapsed(),
                &error_msg,
                None,
                None,
                false,
            );

            return Err(anyhow::anyhow!(error_msg));
        }

        // Get content length from headers before consuming response
        let content_length = response.content_length().unwrap_or(0);

        // Return the response text or handle it as needed
        let content_result = response.text().await;
        let content = match content_result {
            Ok(text) => text,
            Err(e) => {
                self.event_logger.log_crawl_failure(
                    &url,
                    start_time.elapsed(),
                    &format!("Failed to read response body: {}", e),
                    None,
                    None,
                    false,
                );
                return Err(e.into());
            }
        };

        if content.is_empty() {
            self.event_logger.log_crawl_failure(
                &url,
                start_time.elapsed(),
                "Empty response content",
                None,
                None,
                false,
            );
            return Ok(None);
        }

        // 9. Extract and validate content
        let (text, word_count) = match self.content_processor.extract_and_validate(&content).await {
            Ok(result) => result,
            Err(e) => {
                self.event_logger.log_crawl_failure(
                    &url,
                    start_time.elapsed(),
                    &format!("Content processing error: {}", e),
                    None,
                    None,
                    false,
                );
                return Err(e);
            }
        };

        // 10. Return only if meets word count
        if word_count >= self.min_word_length {
            // Log successful crawl
            self.event_logger.log_crawl_success(
                &url,
                start_time.elapsed(),
                status_code,
                content_length,
                word_count,
                None, // Language detection could be added here
                None, // Depth tracking could be added here
                proxy_info,
            );

            Ok(Some(text))
        } else {
            self.event_logger.log_crawl_failure(
                &url,
                start_time.elapsed(),
                &format!(
                    "Content too short: {} words < {} minimum",
                    word_count, self.min_word_length
                ),
                None,
                None,
                false,
            );
            Ok(None)
        }
    }

    /// Get diagnostic information about rate limiting
    pub async fn get_rate_limit_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        let trackers = self.rate_limiter.domain_trackers.read().await;

        for (domain, tracker) in trackers.iter() {
            stats.insert(domain.clone(), tracker.request_timestamps.len());
        }

        stats
    }

    /// Perform periodic maintenance tasks (cleanup caches)
    pub async fn perform_maintenance(&self) {
        self.dns_resolver.cleanup_dns_cache().await;
        // Can be extended with other maintenance tasks
    }

    /// Run concurrent crawling on multiple URLs using futures stream
    pub async fn run_concurrent_crawling(
        seeds: Vec<Url>,
        max_concurrent_tasks: usize,
        config: WebCrawlerConfig,
    ) -> Result<Vec<(Url, Option<String>)>, Error> {
        // Create a single crawler instance
        let crawler = WebCrawler::new(config, max_concurrent_tasks, defaults::DEFAULT_MAX_DEPTH)?;
        let crawler = Arc::new(crawler);

        // Use futures stream with buffer_unordered for concurrency without spawning
        let results: Vec<(Url, Option<String>)> = stream::iter(seeds)
            .map(|url| {
                let crawler = Arc::clone(&crawler);
                let url_clone = url.clone();
                async move {
                    match crawler.init_crawling(url).await {
                        Ok(content) => (url_clone, content),
                        Err(e) => {
                            crawler.event_logger.log_error(
                                crate::logging::ErrorType::NetworkError,
                                &format!("Concurrent crawl error: {}", e),
                                Some(&url_clone),
                                Some("run_concurrent_crawling"),
                            );
                            (url_clone, None)
                        }
                    }
                }
            })
            .buffer_unordered(max_concurrent_tasks)
            .collect()
            .await;

        Ok(results)
    }

    /// Get diagnostic information about DNS cache
    pub async fn get_dns_cache_stats(&self) -> HashMap<String, String> {
        self.dns_resolver.get_dns_cache_stats().await
    }
}

impl HttpClientManager for WebCrawler {
    /// Create HTTP client with cached proxy connections
    async fn create_client_with_proxy(&self) -> Result<Client, Error> {
        if self.proxy_pool.is_empty() {
            return Ok(self.client.clone());
        }

        // Select random proxy
        let mut rng = rand::thread_rng();
        let proxy_url = &self.proxy_pool[rng.gen_range(0..self.proxy_pool.len())];

        // Check if we have a cached client for this proxy
        {
            let clients = self.proxy_clients.lock().await;
            if let Some(cached_client) = clients.get(proxy_url) {
                return Ok(cached_client.clone());
            }
        }

        // Create new client for this proxy
        let proxy = if proxy_url.starts_with("socks5://") {
            Proxy::all(proxy_url)?
        } else {
            Proxy::http(proxy_url)?
        };

        let client = Client::builder()
            .proxy(proxy)
            .redirect(Policy::limited(defaults::MAX_REDIRECTS))
            .timeout(Duration::from_secs(defaults::REQUEST_TIMEOUT_SECS))
            .pool_max_idle_per_host(defaults::CONNECTION_POOL_SIZE)
            .pool_idle_timeout(Duration::from_secs(defaults::CONNECTION_IDLE_TIMEOUT_SECS))
            .build()?;

        // Cache the client
        {
            let mut clients = self.proxy_clients.lock().await;
            clients.insert(proxy_url.clone(), client.clone());
        }

        Ok(client)
    }

    /// Get random User-Agent string
    fn get_random_user_agent(&self) -> &'static str {
        let mut rng = rand::thread_rng();
        defaults::USER_AGENTS[rng.gen_range(0..defaults::USER_AGENTS.len())]
    }

    /// Generate Accept-Language header based on configured accepted languages
    fn get_accept_language_header(&self) -> String {
        if self.accepted_languages.is_empty() {
            return defaults::FALLBACK_ACCEPT_LANGUAGE.to_string();
        }

        let mut language_parts = Vec::new();
        let mut quality = defaults::MAX_QUALITY;
        let quality_step = defaults::QUALITY_STEP_DIVISOR / self.accepted_languages.len() as f64;

        for lang_type in &self.accepted_languages {
            let variants = lang_type.to_http_variants();
            for (i, variant) in variants.iter().enumerate() {
                let q_value = if i == 0 {
                    // First variant gets full quality
                    quality
                } else {
                    // Subsequent variants get slightly lower quality
                    quality - defaults::QUALITY_DECREMENT
                };

                if q_value >= defaults::MAX_QUALITY {
                    language_parts.push(variant.to_string());
                } else {
                    language_parts.push(format!("{};q={:.1}", variant, q_value));
                }
            }
            quality -= quality_step;
        }

        // Add wildcard with low priority
        language_parts.push(defaults::WILDCARD_QUALITY.to_string());
        language_parts.join(",")
    }
}

/*
// TODO: Re-enable once Send issues are resolved with HtmlRewriter and ThreadRng
impl WebCrawler {
    /// Enhanced crawling with message queue for fault tolerance and better concurrency
    pub async fn crawl_with_queue(
        seeds: Vec<Url>,
        max_concurrent_tasks: usize,
        config: WebCrawlerConfig,
        queue_processing_timeout: Option<Duration>,
    ) -> Result<Vec<(Url, Option<String>)>, Error> {
        use crate::queue::{TaskQueue, run_queue_processor};
        use crate::types::{TaskPriority, TaskStatus};

        // Create crawler instance
        let crawler = Arc::new(WebCrawler::new(
            config,
            max_concurrent_tasks,
            defaults::DEFAULT_MAX_DEPTH,
        )?);

        // Create task queue
        let queue = Arc::new(TaskQueue::new(
            max_concurrent_tasks,
            crate::config::defaults::DEFAULT_TASK_QUEUE_RETRIES
        ));

        // Enqueue seed URLs with high priority
        let task_urls: Vec<(Url, TaskPriority)> = seeds
            .iter()
            .map(|url| (url.clone(), TaskPriority::High))
            .collect();

        let _task_ids = queue.enqueue_batch(task_urls).await?;

        // Set up result collection
        let results = Arc::new(Mutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        // Take the receiver from the queue
        let receiver = {
            let mut receiver_opt = queue.result_receiver.write().await;
            receiver_opt.take()
        };

        if let Some(mut receiver) = receiver {
            let results_collector = Arc::clone(&results);
            tokio::spawn(async move {
                while let Some(task_result) = receiver.recv().await {
                    let mut results = results_collector.lock().await;
                    results.push((task_result.url, task_result.content));
                }
            });
        }

        // Create processor function that uses the crawler
        let processor_crawler = Arc::clone(&crawler);
        let processor_fn = move |task: crate::types::CrawlTask| {
            let crawler = Arc::clone(&processor_crawler);
            async move { crawler.init_crawling(task.url).await }
        };

        // Start queue processor
        let queue_clone = Arc::clone(&queue);
        let processor_handle = tokio::spawn(async move {
            if let Err(e) = run_queue_processor(
                queue_clone,
                processor_fn,
                Duration::from_secs(crate::config::defaults::CLEANUP_INTERVAL_SECS), // cleanup interval
            )
            .await
            {
                // Log queue processor error using structured logging
                tracing::error!(
                    error = %e,
                    context = "queue_processor",
                    "Queue processor encountered an error"
                );
            }
        });

        // Wait for processing to complete or timeout
        let timeout_duration = queue_processing_timeout.unwrap_or_else(||
            Duration::from_secs(crate::config::defaults::DEFAULT_QUEUE_PROCESSING_TIMEOUT_SECS)
        ); // 5 minutes default

        let start_time = Instant::now();
        loop {
            let has_work = queue.has_work().await;
            let in_progress = queue.in_progress_count().await;

            if !has_work && in_progress == 0 {
                break; // All work completed
            }

            if start_time.elapsed() > timeout_duration {
                tracing::warn!(
                    timeout_duration_ms = timeout_duration.as_millis(),
                    "Queue processing timeout reached"
                );
                break;
            }

            sleep(Duration::from_millis(500)).await;
        }

        // Stop the processor
        processor_handle.abort();

        // Collect final results
        let final_results = {
            let results = results.lock().await;
            results.clone()
        };

        // Log queue statistics using structured logging
        let stats = queue.get_stats().await;
        tracing::info!(
            completed_tasks = stats.completed_tasks,
            failed_tasks = stats.dead_tasks,
            retried_tasks = stats.retrying_tasks,
            success_rate = %format!("{:.1}%", stats.success_rate),
            avg_processing_time_ms = %format!("{:.1}ms", stats.average_processing_time_ms),
            "Queue Processing Statistics"
        );

        Ok(final_results)
    }

    /// Monitor queue progress and provide real-time statistics
    pub async fn monitor_queue_progress(
        queue: Arc<TaskQueue>,
        update_interval: Duration,
        max_duration: Option<Duration>,
    ) -> Result<(), Error> {
        let start_time = Instant::now();
        let mut last_stats = queue.get_stats().await;

        loop {
            sleep(update_interval).await;

            let current_stats = queue.get_stats().await;
            let has_work = queue.has_work().await;
            let in_progress = queue.in_progress_count().await;

            // Calculate progress
            let total_work = current_stats.total_tasks;
            let completed_work = current_stats.completed_tasks + current_stats.dead_tasks;
            let progress_percent = if total_work > 0 {
                (completed_work as f64 / total_work as f64) * 100.0
            } else {
                0.0
            };

            // Calculate throughput
            let completed_delta = current_stats.completed_tasks - last_stats.completed_tasks;
            let time_delta = update_interval.as_secs_f64();
            let throughput = completed_delta as f64 / time_delta;

            tracing::info!(
                progress_percent = %format!("{:.1}%", progress_percent),
                throughput = %format!("{:.1} tasks/sec", throughput),
                pending_tasks = current_stats.pending_tasks + current_stats.retrying_tasks,
                in_progress = in_progress,
                completed_tasks = current_stats.completed_tasks,
                failed_tasks = current_stats.dead_tasks,
                "Queue Progress Update"
            );

            last_stats = current_stats;

            // Check if processing is complete
            if !has_work && in_progress == 0 {
                tracing::info!("Queue processing completed successfully");
                break;
            }

            // Check timeout
            if let Some(max_duration) = max_duration {
                if start_time.elapsed() > max_duration {
                    tracing::warn!(
                        max_duration_ms = max_duration.as_millis(),
                        "Monitoring timeout reached"
                    );
                    break;
                }
            }
        }

        Ok(())
    }
}
*/
