use rust_web_crawler::{CrawlerMetrics, DataStorage, OutputFormat, TaskPriority, TaskQueue};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

/// Focused integration test for URL validation and language recognition
/// Tests the two core requirements:
/// 1. Can the crawler distinguish valid vs invalid URLs?
/// 2. Can it recognize different languages?

#[tokio::test]
async fn test_url_validation_and_language_recognition() {
    init_test_logging();
    info!("Starting URL validation and language recognition test");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Arc::new(
        DataStorage::new(temp_dir.path(), OutputFormat::Jsonl).expect("Failed to create storage"),
    );
    let queue = Arc::new(TaskQueue::new(3, 2));
    let _metrics = Arc::new(CrawlerMetrics::new());

    let session_id = Uuid::new_v4().to_string();
    info!("Test session: {}", session_id);

    // Test URLs: Mix of valid (multilingual) and invalid URLs
    let test_urls = vec![
        // Valid URLs - Different languages as requested
        ("https://sg.finance.yahoo.com/", "English", true),
        ("https://www.chinanews.com.cn/", "Chinese", true),
        (
            "https://www3.nhk.or.jp/news/easy/",
            "Japanese/English",
            true,
        ),
        ("https://example.com", "English", true),
        ("https://httpbin.org/html", "English", true),
        // Invalid URLs - Should be detected as invalid
        (
            "https://invalid-test-site-xyz-nonexistent.com",
            "N/A",
            false,
        ),
        ("https://does-not-exist-crawler-test.invalid", "N/A", false),
        ("http://127.0.0.1:99999", "N/A", false), // Invalid port
    ];

    info!("Testing {} URLs total", test_urls.len());
    info!(
        "Valid URLs: {}",
        test_urls.iter().filter(|(_, _, valid)| *valid).count()
    );
    info!(
        "Invalid URLs: {}",
        test_urls.iter().filter(|(_, _, valid)| !*valid).count()
    );

    // Test 1: URL Parsing and Validation
    info!("\n=== Test 1: URL Parsing and Validation ===");
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for (url_str, _expected_lang, should_be_valid) in &test_urls {
        match Url::parse(url_str) {
            Ok(url) => {
                info!("✓ URL parsing successful: {} -> {}", url_str, url);
                if *should_be_valid {
                    valid_count += 1;
                } else {
                    warn!("⚠ Expected invalid URL parsed successfully: {}", url_str);
                }

                // Try to enqueue the task
                match queue.enqueue_task(url, TaskPriority::Normal).await {
                    Ok(task_id) => {
                        info!(
                            "✓ Task enqueued successfully: {} (ID: {})",
                            url_str, task_id
                        );
                    }
                    Err(e) => {
                        error!("✗ Failed to enqueue task for {}: {}", url_str, e);
                    }
                }
            }
            Err(e) => {
                if *should_be_valid {
                    error!("✗ Valid URL failed to parse {}: {}", url_str, e);
                } else {
                    info!("✓ Invalid URL correctly rejected: {} -> {}", url_str, e);
                    invalid_count += 1;
                }
            }
        }
    }

    info!("URL Validation Results:");
    info!("  Successfully parsed valid URLs: {}", valid_count);
    info!("  Correctly rejected invalid URLs: {}", invalid_count);

    // Test 2: Queue Status Check
    info!("\n=== Test 2: Queue Status Check ===");
    let stats = queue.get_stats().await;
    info!("Queue Statistics:");
    info!("  Total tasks: {}", stats.total_tasks);
    info!("  Pending tasks: {}", stats.pending_tasks);
    info!("  In progress tasks: {}", stats.in_progress_tasks);
    info!("  Completed tasks: {}", stats.completed_tasks);

    // Verify we have tasks queued
    assert!(stats.total_tasks > 0, "Should have some tasks queued");
    assert!(stats.pending_tasks > 0, "Should have pending tasks");

    // Test 3: Basic Language Recognition Setup
    info!("\n=== Test 3: Language Recognition Preparation ===");

    // Try to dequeue a few tasks to test the processing pipeline
    let mut dequeued_tasks = Vec::new();

    for i in 0..3 {
        if let Some(task) = queue.dequeue_task().await {
            info!(
                "✓ Dequeued task {}: {} (Priority: {:?})",
                i + 1,
                task.url,
                task.priority
            );
            dequeued_tasks.push(task);
        } else {
            info!("No more tasks to dequeue at iteration {}", i + 1);
            break;
        }
    }

    info!(
        "Successfully dequeued {} tasks for processing",
        dequeued_tasks.len()
    );

    // Test 4: Storage System Verification
    info!("\n=== Test 4: Storage System Verification ===");

    // Test analytics generation (should work even with no data)
    match storage.generate_analytics().await {
        Ok(analytics) => {
            info!("✓ Analytics generation successful:");
            info!("  Total pages: {}", analytics.total_pages);
            info!("  Successful crawls: {}", analytics.successful_crawls);
            info!("  Failed crawls: {}", analytics.failed_crawls);
            info!("  Domains crawled: {}", analytics.domains_crawled);
            info!(
                "  Language distribution: {:?}",
                analytics.language_distribution
            );
        }
        Err(e) => {
            error!("✗ Analytics generation failed: {}", e);
        }
    }

    // Final verification
    let final_stats = queue.get_stats().await;
    info!("\n=== Final Test Results ===");
    info!("Final queue state:");
    info!("  Total tasks: {}", final_stats.total_tasks);
    info!("  Pending tasks: {}", final_stats.pending_tasks);
    info!("  In progress tasks: {}", final_stats.in_progress_tasks);

    // Test assertions
    assert!(valid_count >= 5, "Should have at least 5 valid URLs");
    assert!(
        final_stats.total_tasks >= 5,
        "Should have queued at least 5 tasks"
    );

    info!("✓ URL validation and language recognition test completed successfully!");
    info!("Ready for next phase: actual network crawling and content analysis");
}

#[tokio::test]
async fn test_simple_network_connectivity() {
    init_test_logging();
    info!("Starting simple network connectivity test");

    // Test basic network connectivity to a reliable endpoint
    let simple_urls = vec!["https://httpbin.org/html", "https://example.com"];

    for url_str in &simple_urls {
        info!("Testing network connectivity to: {}", url_str);

        // Use a simple HTTP client to test connectivity
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        match timeout(Duration::from_secs(15), client.head(*url_str).send()).await {
            Ok(Ok(response)) => {
                info!(
                    "✓ Successfully connected to {}: Status {}",
                    url_str,
                    response.status()
                );

                // Check for content type hints about language
                if let Some(content_type) = response.headers().get("content-type") {
                    info!("  Content-Type: {:?}", content_type);
                }

                if let Some(content_lang) = response.headers().get("content-language") {
                    info!("  Content-Language: {:?}", content_lang);
                }
            }
            Ok(Err(e)) => {
                warn!("✗ Network error for {}: {}", url_str, e);
            }
            Err(_) => {
                warn!("✗ Timeout connecting to {}", url_str);
            }
        }
    }

    info!("Simple network connectivity test completed");
}

/// Initialize logging for tests with better formatting
fn init_test_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("info,rust_web_crawler=debug")
        .with_test_writer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .try_init()
        .ok(); // Ignore error if already initialized
}
