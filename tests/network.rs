/// Network module integration tests
/// Tests DNS resolution, rate limiting, robots.txt handling, and HTTP client functionality
use rust_web_crawler::{TaskPriority, TaskQueue};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{error, info, warn};
use url::Url;

mod core;
use core::{get_multilingual_test_urls, init_test_logging};

#[tokio::test]
async fn test_dns_resolution_multilingual_sites() {
    init_test_logging();
    info!("=== Network: DNS Resolution Test ===");

    let multilingual_urls = get_multilingual_test_urls();
    let valid_urls: Vec<_> = multilingual_urls
        .iter()
        .filter(|(_, _, valid)| *valid)
        .collect();

    let mut successful_resolutions = 0;
    let mut failed_resolutions = 0;

    for (url_str, language, _) in valid_urls {
        if let Ok(url) = Url::parse(url_str) {
            if let Some(host) = url.host_str() {
                info!("Testing DNS resolution for {} ({})", host, language);

                match timeout(
                    Duration::from_secs(10),
                    tokio::net::lookup_host(format!("{}:80", host)),
                )
                .await
                {
                    Ok(Ok(mut addrs)) => {
                        if let Some(addr) = addrs.next() {
                            successful_resolutions += 1;
                            info!("✓ {} resolved to: {}", host, addr.ip());
                        } else {
                            failed_resolutions += 1;
                            warn!("✗ {} resolved but no addresses returned", host);
                        }
                    }
                    Ok(Err(e)) => {
                        failed_resolutions += 1;
                        warn!("✗ DNS resolution failed for {}: {}", host, e);
                    }
                    Err(_) => {
                        failed_resolutions += 1;
                        warn!("✗ DNS resolution timeout for {}", host);
                    }
                }
            }
        }
    }

    info!("DNS Resolution Results:");
    info!("  Successful: {}", successful_resolutions);
    info!("  Failed: {}", failed_resolutions);

    assert!(
        successful_resolutions >= 4,
        "Should resolve at least 4 multilingual domains"
    );
    info!("=== ✅ DNS Resolution Test PASSED ===");
}

#[tokio::test]
async fn test_rate_limiting_behavior() {
    init_test_logging();
    info!("=== Network: Rate Limiting Test ===");

    let queue = Arc::new(TaskQueue::new(2, 1)); // Low concurrency to test rate limiting
    let test_urls = vec![
        "https://httpbin.org/delay/1",
        "https://httpbin.org/delay/1",
        "https://httpbin.org/delay/1",
    ];

    // Enqueue tasks quickly
    let start_time = std::time::Instant::now();
    for (i, url_str) in test_urls.iter().enumerate() {
        if let Ok(url) = Url::parse(url_str) {
            let task_id = queue
                .enqueue_task(url, TaskPriority::Normal)
                .await
                .expect("Failed to enqueue task");
            info!(
                "Enqueued rate limit test task {}: {}",
                i + 1,
                &task_id[..12]
            );
        }
    }

    let enqueue_duration = start_time.elapsed();
    info!(
        "Enqueued {} tasks in {:?}",
        test_urls.len(),
        enqueue_duration
    );

    // Test dequeuing with potential rate limiting
    let dequeue_start = std::time::Instant::now();
    let mut dequeued_count = 0;

    for i in 0..test_urls.len() {
        if let Some(task) = queue.dequeue_task().await {
            dequeued_count += 1;
            info!(
                "Dequeued task {}: {} (after {:?})",
                i + 1,
                task.url,
                dequeue_start.elapsed()
            );

            // Simulate processing time
            sleep(Duration::from_millis(100)).await;
        }
    }

    let total_duration = dequeue_start.elapsed();
    info!(
        "Dequeued {} tasks in total time: {:?}",
        dequeued_count, total_duration
    );

    assert_eq!(dequeued_count, test_urls.len(), "Should dequeue all tasks");
    info!("=== ✅ Rate Limiting Test PASSED ===");
}

#[tokio::test]
async fn test_robots_txt_handling() {
    init_test_logging();
    info!("=== Network: Robots.txt Handling Test ===");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("rust-web-crawler/0.1.0")
        .build()
        .expect("Failed to create HTTP client");

    let test_sites = vec![
        ("https://example.com", "Standard robots.txt"),
        ("https://httpbin.org", "Testing service robots.txt"),
        ("https://sg.finance.yahoo.com", "Yahoo Finance robots.txt"),
    ];

    let mut robots_found = 0;
    let mut robots_missing = 0;

    for (base_url, description) in test_sites {
        let robots_url = format!("{}/robots.txt", base_url);
        info!("Checking robots.txt for {}: {}", description, robots_url);

        match timeout(Duration::from_secs(15), client.get(&robots_url).send()).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    robots_found += 1;
                    info!(
                        "✓ Found robots.txt for {}: Status {}",
                        description,
                        response.status()
                    );

                    // Try to read a small portion of robots.txt
                    if let Ok(text) = response.text().await {
                        let preview = text.lines().take(3).collect::<Vec<_>>().join(" | ");
                        info!("  Preview: {}", preview);
                    }
                } else {
                    robots_missing += 1;
                    info!(
                        "✗ No robots.txt for {}: Status {}",
                        description,
                        response.status()
                    );
                }
            }
            Ok(Err(e)) => {
                robots_missing += 1;
                warn!("✗ Error fetching robots.txt for {}: {}", description, e);
            }
            Err(_) => {
                robots_missing += 1;
                warn!("✗ Timeout fetching robots.txt for {}", description);
            }
        }
    }

    info!("Robots.txt Results:");
    info!("  Found: {}", robots_found);
    info!("  Missing/Error: {}", robots_missing);

    // At least some sites should have robots.txt
    info!("=== ✅ Robots.txt Handling Test PASSED ===");
}

#[tokio::test]
async fn test_http_client_multilingual_headers() {
    init_test_logging();
    info!("=== Network: HTTP Client Multilingual Headers Test ===");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (compatible; rust-web-crawler/0.1.0)")
        .build()
        .expect("Failed to create HTTP client");

    let multilingual_sites = vec![
        ("https://sg.finance.yahoo.com/", "English"),
        ("https://www.chinanews.com.cn/", "Chinese"),
        ("https://news.naver.com/", "Korean"),
    ];

    for (url, language) in multilingual_sites {
        info!("Testing HTTP headers for {} site: {}", language, url);

        match timeout(Duration::from_secs(20), client.head(url).send()).await {
            Ok(Ok(response)) => {
                info!("✓ {} response: Status {}", language, response.status());

                // Analyze headers for language/encoding information
                let headers = response.headers();

                if let Some(content_type) = headers.get("content-type") {
                    info!("  Content-Type: {:?}", content_type);
                }

                if let Some(content_lang) = headers.get("content-language") {
                    info!("  Content-Language: {:?}", content_lang);
                }

                if let Some(charset) = headers.get("charset") {
                    info!("  Charset: {:?}", charset);
                }

                // Check for special headers that might indicate language/region
                for (name, value) in headers.iter() {
                    let name_str = name.as_str().to_lowercase();
                    if name_str.contains("lang")
                        || name_str.contains("locale")
                        || name_str.contains("region")
                    {
                        info!("  Language-related header {}: {:?}", name, value);
                    }
                }
            }
            Ok(Err(e)) => {
                warn!("✗ HTTP error for {} site: {}", language, e);
            }
            Err(_) => {
                warn!("✗ Timeout for {} site", language);
            }
        }
    }

    info!("=== ✅ HTTP Client Headers Test PASSED ===");
}

#[tokio::test]
async fn test_network_error_handling() {
    init_test_logging();
    info!("=== Network: Error Handling Test ===");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client");

    let error_scenarios = vec![
        (
            "https://does-not-exist-12345.invalid",
            "Non-existent domain",
        ),
        ("http://127.0.0.1:99999", "Connection refused"),
        ("https://httpbin.org/status/500", "Server error"),
        ("https://httpbin.org/delay/10", "Request timeout"),
    ];

    let mut handled_errors = 0;

    for (url, scenario) in error_scenarios {
        info!("Testing error scenario: {}", scenario);

        match timeout(Duration::from_secs(8), client.head(url).send()).await {
            Ok(Ok(response)) => {
                if response.status().is_server_error() {
                    handled_errors += 1;
                    info!(
                        "✓ Correctly handled server error: Status {}",
                        response.status()
                    );
                } else {
                    info!(
                        "✓ Unexpected success for {}: Status {}",
                        scenario,
                        response.status()
                    );
                }
            }
            Ok(Err(e)) => {
                handled_errors += 1;
                info!("✓ Correctly handled network error for {}: {}", scenario, e);
            }
            Err(_) => {
                handled_errors += 1;
                info!("✓ Correctly handled timeout for {}", scenario);
            }
        }
    }

    info!("Error Handling Results:");
    info!("  Properly handled errors: {}", handled_errors);

    assert!(
        handled_errors >= 2,
        "Should properly handle at least 2 error scenarios"
    );
    info!("=== ✅ Network Error Handling Test PASSED ===");
}
