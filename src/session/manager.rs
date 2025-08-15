use anyhow::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::config::WebCrawlerConfig;
use crate::core::types::TaskContent;
use crate::crawler::WebCrawler;
use crate::logging::CrawlEventLogger;
use crate::queue::TaskQueue;
use crate::storage::{DataStorage, StoredCrawlResult};

use super::statistics::SessionStatistics;

/// High-level configuration for a crawl session
#[derive(Debug, Clone)]
pub struct CrawlSessionConfig {
    pub crawler_config: WebCrawlerConfig,
    pub max_concurrent_requests: usize,
    pub max_depth: usize,
    pub max_retries: u32,
    pub session_timeout: Option<Duration>,
    pub enable_storage: bool,
    pub storage_path: Option<String>,
}

impl Default for CrawlSessionConfig {
    fn default() -> Self {
        Self {
            crawler_config: WebCrawlerConfig::default(),
            max_concurrent_requests: 5,
            max_depth: 3,
            max_retries: 3,
            session_timeout: Some(Duration::from_secs(300)), // 5 minutes
            enable_storage: true,
            storage_path: Some("./crawl_data".to_string()),
        }
    }
}

/// Results from a completed crawl session
#[derive(Debug, Clone)]
pub struct SessionResult {
    pub session_id: String,
    pub total_urls_processed: usize,
    pub successful_crawls: usize,
    pub failed_crawls: usize,
    pub total_duration: Duration,
    pub results: Vec<CrawlResultData>,
    pub statistics: SessionStatistics,
}

/// Individual crawl result data
#[derive(Debug, Clone)]
pub struct CrawlResultData {
    pub url: Url,
    pub content: Option<TaskContent>,
    pub error: Option<String>,
    pub duration: Duration,
    pub status_code: Option<u16>,
}

/// High-level crawl session manager that orchestrates the entire crawl process
pub struct CrawlSession {
    session_id: String,
    config: CrawlSessionConfig,
    crawler: Arc<WebCrawler>,
    task_queue: Arc<TaskQueue>,
    event_logger: CrawlEventLogger,
    statistics: Arc<Mutex<SessionStatistics>>,
    storage: Option<DataStorage>,
}

impl CrawlSession {
    /// Create a new crawl session
    pub async fn new(config: CrawlSessionConfig) -> Result<Self, Error> {
        let session_id = Uuid::new_v4().to_string();

        // Create crawler
        let crawler = Arc::new(WebCrawler::new(
            config.crawler_config.clone(),
            config.max_concurrent_requests,
            config.max_depth,
        )?);

        // Create task queue
        let task_queue = Arc::new(TaskQueue::new(
            config.max_concurrent_requests,
            config.max_retries,
        ));

        // Create event logger
        let event_logger = CrawlEventLogger::new(session_id.clone());

        // Create statistics tracker
        let statistics = Arc::new(Mutex::new(SessionStatistics::new()));

        // Create storage if enabled
        let storage = if config.enable_storage {
            let storage_path = config.storage_path.as_deref().unwrap_or("./crawl_data");
            Some(DataStorage::new(
                storage_path,
                crate::storage::OutputFormat::Json,
            )?)
        } else {
            None
        };

        Ok(Self {
            session_id,
            config,
            crawler,
            task_queue,
            event_logger,
            statistics,
            storage,
        })
    }

    /// Execute the crawl session with provided URLs
    pub async fn execute_crawl(&self, urls: Vec<Url>) -> Result<SessionResult, Error> {
        let start_time = Instant::now();

        // Log session start
        tracing::info!(
            session_id = %self.session_id,
            url_count = urls.len(),
            "Starting crawl session"
        );

        // Initialize statistics
        {
            let mut stats = self.statistics.lock().await;
            stats.session_started(urls.len());
        }

        // Enqueue initial URLs
        for url in &urls {
            self.task_queue
                .enqueue_task(url.clone(), crate::core::TaskPriority::High)
                .await?;
        }

        // Process crawl queue
        let results = self.process_crawl_queue().await?;

        let total_duration = start_time.elapsed();

        // Collect final statistics
        let final_stats = {
            let mut stats = self.statistics.lock().await;
            stats.session_completed(total_duration);
            stats.clone()
        };

        // Store results if storage is enabled
        if let Some(storage) = &self.storage {
            self.store_results_to_storage(&results, storage).await?;
        }

        // Log session completion
        tracing::info!(
            session_id = %self.session_id,
            duration_ms = total_duration.as_millis(),
            total_processed = results.len(),
            "Crawl session completed"
        );

        Ok(SessionResult {
            session_id: self.session_id.clone(),
            total_urls_processed: results.len(),
            successful_crawls: results.iter().filter(|r| r.content.is_some()).count(),
            failed_crawls: results.iter().filter(|r| r.content.is_none()).count(),
            total_duration,
            results,
            statistics: final_stats,
        })
    }

    /// Process the crawl queue and collect results
    async fn process_crawl_queue(&self) -> Result<Vec<CrawlResultData>, Error> {
        let mut results = Vec::new();
        let timeout = self
            .config
            .session_timeout
            .unwrap_or(Duration::from_secs(300));
        let start_time = Instant::now();

        while let Some(task) = self.task_queue.dequeue_task().await {
            // Check timeout
            if start_time.elapsed() > timeout {
                tracing::warn!(
                    session_id = %self.session_id,
                    "Session timeout reached, stopping crawl"
                );
                break;
            }

            let url = task.url.clone();
            let task_start = Instant::now();

            // Log crawl start
            self.event_logger
                .log_crawl_start(&url, Some(0), Some("CrawlSession/1.0"));

            // Execute crawl
            match self.crawler.init_crawling(url.clone()).await {
                Ok(Some(content)) => {
                    let duration = task_start.elapsed();
                    let task_content = TaskContent {
                        content: content.clone(),
                        word_count: content.split_whitespace().count(),
                        detected_language: None, // Could be enhanced with language detection
                    };

                    // Complete task in queue
                    let _ = self
                        .task_queue
                        .complete_task(&task.id, Some(content), duration)
                        .await;

                    let result = CrawlResultData {
                        url: url.clone(),
                        content: Some(task_content),
                        error: None,
                        duration,
                        status_code: Some(200),
                    };

                    // Update statistics
                    {
                        let mut stats = self.statistics.lock().await;
                        stats.url_completed(true, duration);
                    }

                    results.push(result);
                }
                Ok(None) => {
                    let duration = task_start.elapsed();

                    // Mark task as failed
                    let _ = self
                        .task_queue
                        .fail_task(&task.id, "No content extracted".to_string(), duration)
                        .await;

                    let result = CrawlResultData {
                        url: url.clone(),
                        content: None,
                        error: Some("No content extracted".to_string()),
                        duration,
                        status_code: None,
                    };

                    // Update statistics
                    {
                        let mut stats = self.statistics.lock().await;
                        stats.url_completed(false, duration);
                    }

                    results.push(result);
                }
                Err(e) => {
                    let duration = task_start.elapsed();

                    // Mark task as failed
                    let _ = self
                        .task_queue
                        .fail_task(
                            &task.id,
                            format!("Network error: {}", e),
                            task_start.elapsed(),
                        )
                        .await;

                    let result = CrawlResultData {
                        url: url.clone(),
                        content: None,
                        error: Some(e.to_string()),
                        duration,
                        status_code: None,
                    };

                    // Update statistics
                    {
                        let mut stats = self.statistics.lock().await;
                        stats.url_completed(false, duration);
                    }

                    results.push(result);
                }
            }

            // Check if queue is empty
            if !self.task_queue.has_work().await {
                break;
            }
        }

        Ok(results)
    }

    /// Store results to configured storage
    async fn store_results_to_storage(
        &self,
        results: &[CrawlResultData],
        storage: &DataStorage,
    ) -> Result<(), Error> {
        for result in results {
            let stored_result = StoredCrawlResult {
                url: result.url.to_string(),
                title: result
                    .content
                    .as_ref()
                    .map(|c| extract_title_from_html(&c.content))
                    .flatten(),
                content: result.content.as_ref().map(|c| c.content.clone()),
                word_count: result.content.as_ref().map(|c| c.word_count).unwrap_or(0),
                language: result
                    .content
                    .as_ref()
                    .and_then(|c| c.detected_language.as_ref())
                    .map(|lang| format!("{:?}", lang)),
                links_found: result
                    .content
                    .as_ref()
                    .map(|c| extract_links_from_html(&c.content))
                    .unwrap_or_default(),
                metadata: crate::storage::CrawlMetadata {
                    status_code: result.status_code,
                    content_type: Some("text/html".to_string()),
                    content_length: result.content.as_ref().map(|c| c.content.len() as u64),
                    response_time_ms: result.duration.as_millis() as u64,
                    depth: 0,
                    parent_url: None,
                    crawl_session_id: self.session_id.clone(),
                },
                timestamp: std::time::SystemTime::now(),
            };

            storage.store_result(&stored_result).await?;
        }

        Ok(())
    }

    /// Get real-time session statistics
    pub async fn get_statistics(&self) -> SessionStatistics {
        self.statistics.lock().await.clone()
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

/// Extract title from HTML content
fn extract_title_from_html(content: &str) -> Option<String> {
    use regex::Regex;

    let re = Regex::new(r"<title[^>]*>([^<]+)</title>").ok()?;
    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Extract links from HTML content
fn extract_links_from_html(content: &str) -> Vec<String> {
    use regex::Regex;

    let re = Regex::new(r#"href\s*=\s*["']([^"']+)["']"#).unwrap();
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|link| link.starts_with("http") || link.starts_with("//"))
        .collect()
}
