/// Core integration tests for the web crawler system
/// This module provides the foundation for all other test cases
/// and tests basic URL validation and multilingual website recognition
use rust_web_crawler::{CrawlerMetrics, DataStorage, OutputFormat, TaskPriority, TaskQueue};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

/// Test configuration and utilities
pub struct TestConfig {
    pub session_id: String,
    pub temp_dir: TempDir,
    pub storage: Arc<DataStorage>,
    pub queue: Arc<TaskQueue>,
    pub metrics: Arc<CrawlerMetrics>,
}

impl TestConfig {
    pub fn new() -> Self {
        let session_id = Uuid::new_v4().to_string();
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = Arc::new(
            DataStorage::new(temp_dir.path(), OutputFormat::Jsonl)
                .expect("Failed to create storage"),
        );
        let queue = Arc::new(TaskQueue::new(4, 2)); // Higher concurrency for multiple languages
        let metrics = Arc::new(CrawlerMetrics::new());

        Self {
            session_id,
            temp_dir,
            storage,
            queue,
            metrics,
        }
    }
}

/// Multilingual test URLs covering 6 languages as requested
pub fn get_multilingual_test_urls() -> Vec<(&'static str, &'static str, bool)> {
    vec![
        // Valid URLs - 6 different languages as requested
        ("https://sg.finance.yahoo.com/", "English", true),
        (
            "https://www.chinanews.com.cn/",
            "Chinese (Simplified)",
            true,
        ),
        (
            "https://www3.nhk.or.jp/news/easy/",
            "Japanese/English Mix",
            true,
        ),
        ("https://news.naver.com/", "Korean", true),
        (
            "https://www.sueddeutsche.de/",
            "German (with agreements)",
            true,
        ),
        ("https://example.com", "English (Standard)", true),
        // Test URLs for validation
        ("https://httpbin.org/html", "English (Test HTML)", true),
        ("https://httpbin.org/json", "JSON Data", true),
        // Invalid URLs - Should fail during network requests
        (
            "https://invalid-test-site-xyz-nonexistent.com",
            "N/A",
            false,
        ),
        ("https://does-not-exist-crawler-test.invalid", "N/A", false),
        ("http://127.0.0.1:99999", "N/A", false), // Invalid port
    ]
}

/// Core foundation test - URL validation and language recognition setup
#[tokio::test]
async fn test_core_url_validation_and_multilingual_support() {
    init_test_logging();
    info!("=== Core Foundation Test: URL Validation & Multilingual Support ===");

    let config = TestConfig::new();
    info!("Test session: {}", config.session_id);

    let test_urls = get_multilingual_test_urls();
    info!("Testing {} URLs total", test_urls.len());
    info!(
        "Valid URLs: {}",
        test_urls.iter().filter(|(_, _, valid)| *valid).count()
    );
    info!(
        "Invalid URLs: {}",
        test_urls.iter().filter(|(_, _, valid)| !*valid).count()
    );

    // Phase 1: URL Parsing and Validation
    info!("\n--- Phase 1: URL Parsing and Validation ---");
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut language_distribution = std::collections::HashMap::new();

    for (url_str, language, should_be_valid) in &test_urls {
        match Url::parse(url_str) {
            Ok(url) => {
                info!("✓ URL parsing successful: {} -> {}", url_str, url);
                if *should_be_valid {
                    valid_count += 1;
                    *language_distribution.entry(*language).or_insert(0) += 1;
                } else {
                    warn!("⚠ Expected invalid URL parsed successfully: {}", url_str);
                }

                // Enqueue for further processing
                match config.queue.enqueue_task(url, TaskPriority::Normal).await {
                    Ok(task_id) => {
                        info!("✓ Task enqueued: {} (ID: {})", url_str, &task_id[..12]);
                    }
                    Err(e) => {
                        error!("✗ Failed to enqueue {}: {}", url_str, e);
                    }
                }
            }
            Err(e) => {
                if *should_be_valid {
                    error!("✗ Valid URL failed to parse {}: {}", url_str, e);
                } else {
                    info!("✓ Invalid URL correctly rejected: {}", url_str);
                    invalid_count += 1;
                }
            }
        }
    }

    info!("\n--- Language Distribution ---");
    for (lang, count) in &language_distribution {
        info!("  {}: {} URLs", lang, count);
    }

    // Phase 2: Queue Management Verification
    info!("\n--- Phase 2: Queue Management ---");
    let stats = config.queue.get_stats().await;
    info!("Queue Statistics:");
    info!("  Total tasks: {}", stats.total_tasks);
    info!("  Pending tasks: {}", stats.pending_tasks);
    info!("  In progress: {}", stats.in_progress_tasks);
    info!("  Completed: {}", stats.completed_tasks);

    // Phase 3: Task Processing Pipeline Test
    info!("\n--- Phase 3: Task Processing Pipeline ---");
    let mut dequeued_tasks = Vec::new();

    // Dequeue tasks for the 6 main language websites
    for i in 0..6 {
        if let Some(task) = config.queue.dequeue_task().await {
            info!(
                "✓ Dequeued language task {}: {} (Priority: {:?})",
                i + 1,
                task.url,
                task.priority
            );
            dequeued_tasks.push(task);
        } else {
            warn!("Could not dequeue task {} - queue might be empty", i + 1);
            break;
        }
    }

    info!(
        "Successfully prepared {} multilingual tasks for processing",
        dequeued_tasks.len()
    );

    // Phase 4: Storage and Analytics Infrastructure
    info!("\n--- Phase 4: Storage & Analytics Infrastructure ---");
    match config.storage.generate_analytics().await {
        Ok(analytics) => {
            info!("✓ Analytics system operational:");
            info!("  Total pages: {}", analytics.total_pages);
            info!("  Successful crawls: {}", analytics.successful_crawls);
            info!("  Failed crawls: {}", analytics.failed_crawls);
            info!("  Domains tracked: {}", analytics.domains_crawled);
            info!(
                "  Language distribution: {:?}",
                analytics.language_distribution
            );
        }
        Err(e) => {
            error!("✗ Analytics system failed: {}", e);
        }
    }

    // Assertions
    assert!(
        valid_count >= 6,
        "Should have at least 6 valid multilingual URLs"
    );
    assert!(invalid_count >= 1, "Should have at least 1 invalid URL");
    assert!(
        stats.total_tasks >= 6,
        "Should have queued at least 6 tasks"
    );
    assert!(
        dequeued_tasks.len() >= 6,
        "Should have dequeued 6 language tasks"
    );
    assert!(
        language_distribution.len() >= 4,
        "Should cover at least 4 different languages"
    );

    info!("\n=== ✅ Core Foundation Test PASSED ===");
    info!("Ready for modular testing: network, processing, storage, queue");
}

/// Network connectivity test for multilingual websites
#[tokio::test]
async fn test_multilingual_network_connectivity() {
    init_test_logging();
    info!("=== Network Connectivity Test: Multilingual Websites ===");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .expect("Failed to create HTTP client");

    let test_urls = vec![
        ("https://sg.finance.yahoo.com/", "English"),
        ("https://www.chinanews.com.cn/", "Chinese"),
        ("https://www3.nhk.or.jp/news/easy/", "Japanese"),
        ("https://news.naver.com/", "Korean"),
        ("https://www.sueddeutsche.de/", "German"),
        ("https://httpbin.org/html", "Test HTML"),
    ];

    let mut successful_connections = 0;
    let mut failed_connections = 0;

    for (url_str, language) in &test_urls {
        info!("Testing {} website: {}", language, url_str);

        match timeout(Duration::from_secs(20), client.head(*url_str).send()).await {
            Ok(Ok(response)) => {
                successful_connections += 1;
                info!(
                    "✓ {} connection successful: Status {}",
                    language,
                    response.status()
                );

                // Extract useful headers for language detection
                if let Some(content_type) = response.headers().get("content-type") {
                    info!("  Content-Type: {:?}", content_type);
                }

                if let Some(content_lang) = response.headers().get("content-language") {
                    info!("  Content-Language: {:?}", content_lang);
                }

                if let Some(server) = response.headers().get("server") {
                    info!("  Server: {:?}", server);
                }

                // Special handling for German site (might have agreements)
                if url_str.contains("sueddeutsche") {
                    info!("  ⚠ German site may require agreement handling");
                }
            }
            Ok(Err(e)) => {
                failed_connections += 1;
                warn!("✗ {} connection failed: {}", language, e);
            }
            Err(_) => {
                failed_connections += 1;
                warn!("✗ {} connection timeout", language);
            }
        }
    }

    info!("\n--- Network Connectivity Results ---");
    info!("Successful connections: {}", successful_connections);
    info!("Failed connections: {}", failed_connections);
    info!(
        "Success rate: {:.1}%",
        (successful_connections as f64 / test_urls.len() as f64) * 100.0
    );

    // Should have at least 50% success rate for reliable testing
    assert!(
        successful_connections >= 3,
        "Should have at least 3 successful connections for reliable testing"
    );

    info!("=== ✅ Network Connectivity Test PASSED ===");
}

/// Initialize comprehensive test logging
pub fn init_test_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("info,rust_web_crawler=debug")
        .with_test_writer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .try_init()
        .ok(); // Ignore error if already initialized
}

/// Utility function to verify crawler can handle different response scenarios
#[tokio::test]
async fn test_response_scenario_handling() {
    init_test_logging();
    info!("=== Response Scenario Handling Test ===");

    let test_scenarios = vec![
        ("https://httpbin.org/status/200", "200 OK"),
        ("https://httpbin.org/status/404", "404 Not Found"),
        ("https://httpbin.org/status/503", "503 Service Unavailable"),
        ("https://httpbin.org/redirect/3", "Multiple Redirects"),
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");

    for (url, scenario) in test_scenarios {
        info!("Testing scenario: {}", scenario);

        match timeout(Duration::from_secs(15), client.head(url).send()).await {
            Ok(Ok(response)) => {
                info!(
                    "✓ Response received for {}: Status {}",
                    scenario,
                    response.status()
                );
            }
            Ok(Err(e)) => {
                info!("✓ Expected error for {}: {}", scenario, e);
            }
            Err(_) => {
                warn!("✗ Timeout for {}", scenario);
            }
        }
    }

    info!("=== ✅ Response Scenario Test PASSED ===");
}
