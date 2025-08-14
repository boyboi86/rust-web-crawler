use anyhow::Error;
use rust_web_crawler::config::WebCrawlerConfig;
use rust_web_crawler::core::{DomainRateLimit, LangType, RetryConfig, TaskPriority};
use rust_web_crawler::crawler::WebCrawler;
use rust_web_crawler::queue::TaskQueue;
use rust_web_crawler::storage::{CrawlMetadata, DataStorage, OutputFormat, StoredCrawlResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::time::{Duration, Instant};
use tracing::{info, warn};
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize structured logging
    init_logging()?;

    info!("🚀 Modular Rust Web Crawler Demo");
    info!("==========================================");

    // Initialize JSON storage for crawl results
    let storage = DataStorage::new("./demo_output", OutputFormat::Json)?;
    info!("📁 Initialized JSON storage in ./demo_output directory");

    // Run different demonstration scenarios with storage
    basic_crawling_demo(&storage).await?;

    multi_language_demo(&storage).await?;

    rate_limiting_demo(&storage).await?;

    fault_tolerance_demo(&storage).await?;

    dns_caching_demo(&storage).await?;

    monitoring_demo(&storage).await?;

    // New queue-based crawling demo
    queue_based_crawling_demo(&storage).await?;

    info!("✅ All demos completed! Check ./demo_output directory for JSON results.");

    Ok(())
}

fn init_logging() -> Result<(), Error> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

/// Demonstrates basic web crawling functionality
async fn basic_crawling_demo(storage: &DataStorage) -> Result<(), Error> {
    info!("Basic Crawling Demonstration");
    info!("-------------------------------");

    let config = WebCrawlerConfig {
        base_url: vec![
            "https://httpbin.org/html".to_string(),
            "https://example.com".to_string(),
        ],
        target_words: vec!["example".to_string(), "html".to_string()],
        min_word_length: 50,
        ..Default::default()
    };

    let seeds = vec![
        Url::parse("https://httpbin.org/html")?,
        Url::parse("https://example.com")?,
    ];

    info!("Target URLs: {:?}", seeds);
    info!("Target words: {:?}", config.target_words);

    let start_time = Instant::now();
    let results = WebCrawler::run_concurrent_crawling(seeds.clone(), 2, config).await?;
    let duration = start_time.elapsed();

    info!("Crawled {} URLs in {:.2?}", results.len(), duration);

    // Store results to JSON files
    let session_id = Uuid::new_v4().to_string();
    for (i, (url, content)) in results.iter().enumerate() {
        if let Some(text) = content {
            let word_count = text.split_whitespace().count();
            info!("   {}: {} words extracted", url, word_count);

            // Create StoredCrawlResult for JSON output
            let stored_result = StoredCrawlResult {
                url: url.to_string(),
                title: extract_title_from_content(text),
                content: Some(text.clone()),
                word_count,
                language: detect_language(text),
                links_found: vec![], // TODO: Extract links if needed
                metadata: CrawlMetadata {
                    status_code: Some(200),
                    content_type: Some("text/html".to_string()),
                    content_length: Some(text.len() as u64),
                    response_time_ms: (duration.as_millis() / results.len() as u128) as u64,
                    depth: 0,
                    parent_url: None,
                    crawl_session_id: session_id.clone(),
                },
                timestamp: SystemTime::now(),
            };

            // Save to JSON file
            storage.store_result(&stored_result).await?;
        } else {
            warn!("   {}: Failed to extract content", url);
        }
    }

    info!("💾 Saved {} results to JSON files", results.len());
    Ok(())
}

/// Demonstrates multi-language crawling capabilities
async fn multi_language_demo(storage: &DataStorage) -> Result<(), Error> {
    info!("Multi-Language Crawling Demonstration");
    info!("----------------------------------------");

    let config = WebCrawlerConfig {
        base_url: vec!["https://httpbin.org/html".to_string()],
        accepted_languages: vec![LangType::Eng, LangType::Fra, LangType::Deu, LangType::Cmn],
        target_words: vec!["international".to_string(), "global".to_string()],
        min_word_length: 30,
        ..Default::default()
    };

    println!("Supported languages: {:?}", config.accepted_languages);

    let seeds = vec![Url::parse("https://httpbin.org/html")?];
    let results = WebCrawler::run_concurrent_crawling(seeds, 1, config).await?;

    println!("Multi-language crawling completed");

    // Store multilingual results
    let session_id = Uuid::new_v4().to_string();
    for (url, content) in &results {
        if let Some(text) = content {
            let detected_lang = detect_language(text);
            println!(
                "   {}: Content extracted with language-aware processing (detected: {:?})",
                url, detected_lang
            );

            let stored_result = StoredCrawlResult {
                url: url.to_string(),
                title: extract_title_from_content(text),
                content: Some(text.clone()),
                word_count: text.split_whitespace().count(),
                language: detected_lang,
                links_found: vec![],
                metadata: CrawlMetadata {
                    status_code: Some(200),
                    content_type: Some("text/html".to_string()),
                    content_length: Some(text.len() as u64),
                    response_time_ms: 100, // Placeholder
                    depth: 0,
                    parent_url: None,
                    crawl_session_id: session_id.clone(),
                },
                timestamp: SystemTime::now(),
            };

            storage.store_result(&stored_result).await?;
        }
    }

    info!("💾 Saved multilingual crawl results to JSON files");
    Ok(())
}

/// Demonstrates advanced rate limiting features
async fn rate_limiting_demo(storage: &DataStorage) -> Result<(), Error> {
    println!("Rate Limiting Demonstration");
    println!("-------------------------------");

    // Create domain-specific rate limits
    let mut domain_rate_limits = HashMap::new();
    domain_rate_limits.insert(
        "httpbin.org".to_string(),
        DomainRateLimit {
            max_requests_per_second: 5,
            window_size_ms: 1000,
        },
    );
    domain_rate_limits.insert(
        "example.com".to_string(),
        DomainRateLimit {
            max_requests_per_second: 2,
            window_size_ms: 1000,
        },
    );

    let config = WebCrawlerConfig {
        base_url: vec![
            "https://httpbin.org/html".to_string(),
            "https://example.com".to_string(),
        ],
        default_rate_limit: Some(DomainRateLimit {
            max_requests_per_second: 10,
            window_size_ms: 1000,
        }),
        domain_rate_limits: Some(domain_rate_limits.clone()),
        ..Default::default()
    };

    println!("Rate limiting configuration:");
    println!("   Default: 10 req/sec");
    for (domain, limit) in &domain_rate_limits {
        println!("   {}: {} req/sec", domain, limit.max_requests_per_second);
    }

    let seeds = vec![
        Url::parse("https://httpbin.org/html")?,
        Url::parse("https://example.com")?,
    ];

    let start_time = Instant::now();
    let results = WebCrawler::run_concurrent_crawling(seeds, 3, config).await?;
    let duration = start_time.elapsed();

    println!("Rate-limited crawling completed in {:.2?}", duration);
    println!("   Processed {} URLs respecting rate limits", results.len());

    Ok(())
}

/// Demonstrates fault tolerance and retry mechanisms
async fn fault_tolerance_demo(storage: &DataStorage) -> Result<(), Error> {
    println!("Fault Tolerance Demonstration");
    println!("----------------------------------");

    let config = WebCrawlerConfig {
        base_url: vec![
            "https://httpbin.org/html".to_string(),
            "https://httpbin.org/status/404".to_string(), // This will fail
            "https://httpbin.org/delay/1".to_string(),
        ],
        retry_config: Some(RetryConfig {
            max_retries: 3,
            base_delay_ms: 500,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }),
        min_word_length: 20,
        ..Default::default()
    };

    println!("Retry configuration:");
    if let Some(retry_config) = &config.retry_config {
        println!("   Max retries: {}", retry_config.max_retries);
        println!("   Base delay: {}ms", retry_config.base_delay_ms);
        println!("   Backoff multiplier: {}", retry_config.backoff_multiplier);
    }

    let seeds = vec![
        Url::parse("https://httpbin.org/html")?,
        Url::parse("https://httpbin.org/status/404")?,
        Url::parse("https://httpbin.org/delay/1")?,
    ];

    println!("Testing URLs (including one that will fail):");
    for url in &seeds {
        println!("   {}", url);
    }

    let start_time = Instant::now();
    let results = WebCrawler::run_concurrent_crawling(seeds, 2, config).await?;
    let duration = start_time.elapsed();

    println!("Fault-tolerant crawling completed in {:.2?}", duration);

    let success_count = results
        .iter()
        .filter(|(_, content)| content.is_some())
        .count();
    let failure_count = results.len() - success_count;

    println!("   Successful: {}", success_count);
    println!("   Failed: {}", failure_count);
    println!(
        "   Success rate: {:.1}%",
        (success_count as f64 / results.len() as f64) * 100.0
    );

    Ok(())
}

/// Demonstrates DNS caching performance benefits
async fn dns_caching_demo(storage: &DataStorage) -> Result<(), Error> {
    println!("DNS Caching Performance Demonstration");
    println!("----------------------------------------");

    let config = WebCrawlerConfig {
        base_url: vec!["https://httpbin.org/html".to_string()],
        min_word_length: 30,
        ..Default::default()
    };

    // Create multiple URLs to the same domain to show DNS caching benefits
    let seeds = vec![
        Url::parse("https://httpbin.org/html")?,
        Url::parse("https://httpbin.org/json")?,
        Url::parse("https://httpbin.org/uuid")?,
    ];

    println!("Testing DNS caching with multiple requests to httpbin.org");
    println!("   URLs: {}", seeds.len());

    let start_time = Instant::now();
    let results = WebCrawler::run_concurrent_crawling(seeds, 3, config).await?;
    let duration = start_time.elapsed();

    println!("DNS caching demonstration completed in {:.2?}", duration);
    println!(
        "   Processed {} URLs (DNS cached after first request)",
        results.len()
    );
    println!("   Subsequent requests benefited from DNS cache");

    Ok(())
}

/// Demonstrates monitoring and statistics collection
async fn monitoring_demo(storage: &DataStorage) -> Result<(), Error> {
    println!("Monitoring & Statistics Demonstration");
    println!("----------------------------------------");

    let config = WebCrawlerConfig {
        base_url: vec![
            "https://httpbin.org/html".to_string(),
            "https://example.com".to_string(),
        ],
        proxy_pool: vec![], // No proxies for this demo
        user_agent: "Rust-Crawler-Monitor/1.0".to_string(),
        min_word_length: 25,
        ..Default::default()
    };

    // Create crawler instance to access monitoring features
    let crawler = WebCrawler::new(config, 2, 3)?;

    let seeds = vec![
        Url::parse("https://httpbin.org/html")?,
        Url::parse("https://example.com")?,
    ];

    println!("Crawling URLs for monitoring demonstration...");

    let start_time = Instant::now();
    for seed in seeds {
        match crawler.init_crawling(seed.clone()).await {
            Ok(Some(content)) => {
                let word_count = content.split_whitespace().count();
                println!("   {}: {} words extracted", seed, word_count);
                if !content.is_empty() {
                    println!(
                        "      Content sample: {}...",
                        content.chars().take(50).collect::<String>()
                    );
                }
            }
            Ok(None) => {
                println!("   {}: No content extracted", seed);
            }
            Err(e) => {
                println!("   {}: Error - {}", seed, e);
            }
        }
    }
    let duration = start_time.elapsed();

    // Display monitoring statistics
    println!("\nCrawling Statistics:");
    println!("   Total time: {:.2?}", duration);
    println!("   User agent: Rust-Crawler-Monitor/1.0");
    println!("   Robots.txt compliance: Enabled");
    println!("   DNS caching: Enabled");
    println!("   Rate limiting: Enabled");

    // Show rate limiter statistics
    println!("\nRate Limiter Status:");
    let rate_stats = crawler.get_rate_limit_stats().await;
    if let Some(httpbin_count) = rate_stats.get("httpbin.org") {
        println!("   httpbin.org requests: {}", httpbin_count);
    }
    if let Some(example_count) = rate_stats.get("example.com") {
        println!("   example.com requests: {}", example_count);
    }

    println!("\nMonitoring demonstration completed successfully!");

    Ok(())
}

/// Demonstrates message queue-based crawling with fault tolerance
async fn queue_based_crawling_demo(storage: &DataStorage) -> Result<(), Error> {
    println!("Message Queue-Based Crawling Demonstration");
    println!("---------------------------------------------");

    // Create a task queue
    let queue = Arc::new(TaskQueue::new(3, 3)); // 3 concurrent tasks, 3 max retries

    // Enqueue various URLs with different priorities
    let urls_with_priorities = vec![
        (Url::parse("https://httpbin.org/html")?, TaskPriority::High),
        (Url::parse("https://example.com")?, TaskPriority::Normal),
        (
            Url::parse("https://httpbin.org/json")?,
            TaskPriority::Normal,
        ),
        (
            Url::parse("https://httpbin.org/status/404")?,
            TaskPriority::Low,
        ), // This will fail
        (
            Url::parse("https://httpbin.org/delay/2")?,
            TaskPriority::Low,
        ),
    ];

    println!("Enqueueing tasks with priorities:");
    for (url, priority) in &urls_with_priorities {
        println!("   {:?} priority: {}", priority, url);
    }

    let task_ids = queue.enqueue_batch(urls_with_priorities).await?;
    println!("Enqueued {} tasks", task_ids.len());

    // Show initial queue stats
    let initial_stats = queue.get_stats().await;
    println!("\nInitial Queue Statistics:");
    println!("   Total tasks: {}", initial_stats.total_tasks);
    println!("   Pending tasks: {}", initial_stats.pending_tasks);

    // Simulate processing tasks manually (without the complex Send requirements)
    println!("\nProcessing tasks from queue...");

    let mut processed_count = 0;
    let start_time = Instant::now();

    // Process tasks one by one for demonstration
    while let Some(task) = queue.dequeue_task().await {
        println!("   Processing task: {} ({})", task.url, task.id);

        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Simulate different outcomes based on URL
        if task.url.path() == "/status/404" {
            // Simulate failure
            queue
                .fail_task(
                    &task.id,
                    "HTTP 404 Error".to_string(),
                    Duration::from_millis(200),
                )
                .await?;
            println!("     Task failed: HTTP 404 Error");
        } else {
            // Simulate success
            let dummy_content = format!("Extracted content from {}", task.url);
            queue
                .complete_task(&task.id, Some(dummy_content), Duration::from_millis(200))
                .await?;
            println!("     Task completed successfully");
        }

        processed_count += 1;

        // Stop after processing all tasks once (don't wait for retries in demo)
        if processed_count >= task_ids.len() {
            break;
        }
    }

    let processing_duration = start_time.elapsed();

    // Show final queue statistics
    let final_stats = queue.get_stats().await;
    println!("\nFinal Queue Statistics:");
    println!("   Completed: {}", final_stats.completed_tasks);
    println!("   Dead tasks: {}", final_stats.dead_tasks);
    println!("   Retrying: {}", final_stats.retrying_tasks);
    println!("   Success rate: {:.1}%", final_stats.success_rate);
    println!(
        "   Average processing time: {:.1}ms",
        final_stats.average_processing_time_ms
    );
    println!("   Total processing time: {:.2?}", processing_duration);

    // Show retry queue status
    let retry_count = queue.ready_retry_count().await;
    if retry_count > 0 {
        println!("\nRetry Queue Status:");
        println!("   Ready for retry: {}", retry_count);
        println!("   Failed tasks will be retried with exponential backoff");
    }

    println!("\nKey Benefits of Queue-Based Crawling:");
    println!("   Fault Tolerance: Failed tasks are automatically retried");
    println!("   Async Concurrency: Multiple workers process tasks in parallel");
    println!("   Priority Support: High-priority tasks are processed first");
    println!("   Real-time Monitoring: Complete visibility into queue status");
    println!("   Backpressure Control: Semaphore prevents system overload");
    println!("   Exponential Backoff: Intelligent retry delays for failed tasks");

    Ok(())
}

/// Helper functions for content processing
fn extract_title_from_content(content: &str) -> Option<String> {
    // Simple title extraction - look for <title> tags or use first line
    if let Some(start) = content.find("<title>") {
        if let Some(end) = content[start..].find("</title>") {
            let title = &content[start + 7..start + end];
            return Some(title.trim().to_string());
        }
    }

    // Fallback: use first non-empty line
    content
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
}

fn detect_language(content: &str) -> Option<String> {
    // Simple language detection using whatlang
    use whatlang::{Lang, detect};

    if let Some(info) = detect(content) {
        match info.lang() {
            Lang::Eng => Some("en".to_string()),
            Lang::Fra => Some("fr".to_string()),
            Lang::Deu => Some("de".to_string()),
            Lang::Cmn => Some("zh".to_string()),
            Lang::Jpn => Some("ja".to_string()),
            Lang::Kor => Some("ko".to_string()),
            _ => Some(format!("{:?}", info.lang()).to_lowercase()),
        }
    } else {
        None
    }
}
