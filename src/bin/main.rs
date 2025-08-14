use anyhow::Error;
use rust_web_crawler::config::WebCrawlerConfig;
use rust_web_crawler::core::{DomainRateLimit, LangType, RetryConfig};
use rust_web_crawler::crawler::WebCrawler;
use rust_web_crawler::logging::{CrawlEventLogger, ErrorType};
use rust_web_crawler::storage::{CrawlMetadata, DataStorage, OutputFormat, StoredCrawlResult};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::time::Instant;
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize structured logging
    init_logging()?;

    info!("🚀 Rust Web Crawler - Production Mode");
    info!("=====================================");

    // Initialize JSON storage for crawl results (default output format)
    let storage = DataStorage::new("./crawl_data", OutputFormat::Json)?;
    info!("📁 Initialized JSON storage in ./crawl_data directory");

    // Initialize structured logging for crawl events
    let event_logger = CrawlEventLogger::new("json".to_string());

    // Create production-ready configuration with sensible defaults
    let config = create_production_config();
    info!("⚙️ Loaded production configuration");

    // Define target URLs for crawling
    let target_urls = get_default_crawl_targets();
    info!("🎯 Target URLs loaded: {} sites", target_urls.len());

    // Create session ID for this crawl session
    let session_id = Uuid::new_v4().to_string();
    info!("📝 Session ID: {}", session_id);

    // Log crawl session start
    event_logger
        .log_session_start(&session_id, &target_urls)
        .await;

    // Execute the crawl
    let crawl_results =
        execute_crawl_session(&config, target_urls, &session_id, &event_logger).await?;

    // Store all results to JSON files
    store_crawl_results(&storage, &crawl_results, &session_id).await?;

    // Generate and save session summary
    let summary = generate_session_summary(&session_id, &crawl_results);
    storage.store_session_summary(&session_id, &summary).await?;

    // Log final statistics
    log_final_statistics(&crawl_results);

    // Log crawl session completion
    event_logger
        .log_session_complete(&session_id, crawl_results.len())
        .await;

    info!("✅ Crawl session completed successfully!");
    info!("📊 Results saved to: ./crawl_data directory");
    info!("💾 Output format: JSON (one file per URL + session summary)");

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

fn create_production_config() -> WebCrawlerConfig {
    // Create domain-specific rate limits for common sites
    let mut domain_rate_limits = HashMap::new();

    // Conservative rate limits for news sites
    domain_rate_limits.insert(
        "bbc.com".to_string(),
        DomainRateLimit {
            max_requests_per_second: 2,
            window_size_ms: 1000,
        },
    );
    domain_rate_limits.insert(
        "news.naver.com".to_string(),
        DomainRateLimit {
            max_requests_per_second: 1,
            window_size_ms: 2000,
        },
    );
    domain_rate_limits.insert(
        "spiegel.de".to_string(),
        DomainRateLimit {
            max_requests_per_second: 1,
            window_size_ms: 2000,
        },
    );

    // Faster for test sites
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
            max_requests_per_second: 3,
            window_size_ms: 1000,
        },
    );

    WebCrawlerConfig {
        base_url: vec![
            "https://www.bbc.com".to_string(),
            "https://news.naver.com".to_string(),
            "https://httpbin.org".to_string(),
            "https://example.com".to_string(),
        ],
        accepted_languages: vec![
            LangType::Eng, // English
            LangType::Kor, // Korean
            LangType::Deu, // German
            LangType::Fra, // French
            LangType::Cmn, // Chinese
            LangType::Jpn, // Japanese
        ],
        target_words: vec![
            "news".to_string(),
            "article".to_string(),
            "content".to_string(),
            "information".to_string(),
        ],
        min_word_length: 100, // Ensure we get substantial content
        user_agent: "Rust-Web-Crawler/1.0 (Educational Purpose)".to_string(),
        default_rate_limit: Some(DomainRateLimit {
            max_requests_per_second: 3,
            window_size_ms: 1000,
        }),
        domain_rate_limits: Some(domain_rate_limits),
        retry_config: Some(RetryConfig {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.2,
        }),
        proxy_pool: vec![],           // No proxies in default setup
        avoid_url_extensions: vec![], // No extensions to avoid
    }
}

fn get_default_crawl_targets() -> Vec<Url> {
    vec![
        // English content
        Url::parse("https://www.bbc.com/news").unwrap(),
        Url::parse("https://httpbin.org/html").unwrap(),
        Url::parse("https://example.com").unwrap(),
        // Korean content
        Url::parse("https://news.naver.com").unwrap(),
        // Test different content types
        Url::parse("https://httpbin.org/json").unwrap(),
    ]
}

async fn execute_crawl_session(
    config: &WebCrawlerConfig,
    urls: Vec<Url>,
    session_id: &str,
    event_logger: &CrawlEventLogger,
) -> Result<Vec<CrawlResult>, Error> {
    info!("🕷️ Starting crawl session with {} URLs", urls.len());

    let start_time = Instant::now();

    // Create crawler instance
    let crawler = WebCrawler::new(config.clone(), 3, 3)?; // 3 concurrent, depth 3

    let mut results = Vec::new();

    // Process each URL individually for better error handling and logging
    for (index, url) in urls.iter().enumerate() {
        info!("🌐 Crawling URL {}/{}: {}", index + 1, urls.len(), url);

        // Log crawl start for this URL
        event_logger.log_crawl_start(url, Some(0), Some("Rust-Web-Crawler/1.0"));

        let url_start_time = Instant::now();

        match crawler.init_crawling(url.clone()).await {
            Ok(Some(content)) => {
                let duration = url_start_time.elapsed();
                let word_count = content.split_whitespace().count();
                let language = detect_language(&content);

                info!(
                    "✅ Success: {} chars, {} words, language: {:?}",
                    content.len(),
                    word_count,
                    language
                );

                // Log successful content extraction
                event_logger
                    .log_content_extracted(
                        url,
                        content.len(),
                        &language.unwrap_or_else(|| "unknown".to_string()),
                    )
                    .await;

                results.push(CrawlResult {
                    url: url.clone(),
                    content: Some(content),
                    word_count,
                    language,
                    duration,
                    status_code: Some(200),
                    error: None,
                });
            }
            Ok(None) => {
                let duration = url_start_time.elapsed();
                warn!("⚠️  No content extracted from {}", url);

                // Log crawl skipped
                event_logger
                    .log_crawl_skipped(url, "No content extracted")
                    .await;

                results.push(CrawlResult {
                    url: url.clone(),
                    content: None,
                    word_count: 0,
                    language: None,
                    duration,
                    status_code: None,
                    error: Some("No content extracted".to_string()),
                });
            }
            Err(e) => {
                let duration = url_start_time.elapsed();
                error!("❌ Failed to crawl {}: {}", url, e);

                // Log crawl error
                event_logger.log_error(
                    ErrorType::NetworkError,
                    &e.to_string(),
                    Some(url),
                    Some("Failed to crawl URL"),
                );

                results.push(CrawlResult {
                    url: url.clone(),
                    content: None,
                    word_count: 0,
                    language: None,
                    duration,
                    status_code: None,
                    error: Some(e.to_string()),
                });
            }
        }

        // Small delay between requests to be respectful
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    let total_duration = start_time.elapsed();
    info!("🏁 Crawl session completed in {:.2?}", total_duration);

    Ok(results)
}

async fn store_crawl_results(
    storage: &DataStorage,
    results: &[CrawlResult],
    session_id: &str,
) -> Result<(), Error> {
    info!("💾 Storing {} crawl results to JSON files", results.len());

    for (index, result) in results.iter().enumerate() {
        let stored_result = StoredCrawlResult {
            url: result.url.to_string(),
            title: result.content.as_ref().and_then(extract_title_from_content),
            content: result.content.clone(),
            word_count: result.word_count,
            language: result.language.clone(),
            links_found: result
                .content
                .as_ref()
                .map(extract_links_from_content)
                .unwrap_or_else(Vec::new),
            metadata: CrawlMetadata {
                status_code: result.status_code,
                content_type: Some("text/html".to_string()),
                content_length: result.content.as_ref().map(|c| c.len() as u64),
                response_time_ms: result.duration.as_millis() as u64,
                depth: 0,
                parent_url: None,
                crawl_session_id: session_id.to_string(),
            },
            timestamp: SystemTime::now(),
        };

        // Store each result as a separate JSON file
        storage.store_result(&stored_result).await?;

        info!(
            "   📄 Saved result {}/{}: {}",
            index + 1,
            results.len(),
            result.url
        );
    }

    Ok(())
}

fn generate_session_summary(
    session_id: &str,
    results: &[CrawlResult],
) -> rust_web_crawler::storage::CrawlSessionSummary {
    let successful_crawls = results.iter().filter(|r| r.content.is_some()).count();
    let failed_crawls = results.len() - successful_crawls;
    let total_bytes: u64 = results
        .iter()
        .filter_map(|r| r.content.as_ref())
        .map(|content| content.len() as u64)
        .sum();

    let unique_domains = results
        .iter()
        .filter_map(|r| r.url.host_str())
        .collect::<std::collections::HashSet<_>>()
        .len();

    rust_web_crawler::storage::CrawlSessionSummary {
        session_id: session_id.to_string(),
        start_time: SystemTime::now(),
        end_time: SystemTime::now(),
        total_urls_processed: results.len(),
        successful_crawls,
        failed_crawls,
        total_bytes_downloaded: total_bytes,
        unique_domains,
        configuration: "Production configuration with multilingual support".to_string(),
    }
}

fn log_final_statistics(results: &[CrawlResult]) {
    let successful = results.iter().filter(|r| r.content.is_some()).count();
    let failed = results.len() - successful;
    let total_words: usize = results.iter().map(|r| r.word_count).sum();
    let avg_words = if successful > 0 {
        total_words / successful
    } else {
        0
    };

    let languages: std::collections::HashMap<String, usize> = results
        .iter()
        .filter_map(|r| r.language.as_ref())
        .fold(std::collections::HashMap::new(), |mut acc, lang| {
            *acc.entry(lang.clone()).or_insert(0) += 1;
            acc
        });

    info!("📊 Final Crawl Statistics:");
    info!("   Total URLs processed: {}", results.len());
    info!("   Successful crawls: {}", successful);
    info!("   Failed crawls: {}", failed);
    info!(
        "   Success rate: {:.1}%",
        (successful as f64 / results.len() as f64) * 100.0
    );
    info!("   Total words extracted: {}", total_words);
    info!("   Average words per page: {}", avg_words);

    if !languages.is_empty() {
        info!("   Languages detected:");
        for (lang, count) in &languages {
            info!("     {}: {} pages", lang, count);
        }
    }
}

// Helper structures and functions

#[derive(Debug)]
struct CrawlResult {
    url: Url,
    content: Option<String>,
    word_count: usize,
    language: Option<String>,
    duration: tokio::time::Duration,
    status_code: Option<u16>,
    error: Option<String>,
}

fn extract_title_from_content(content: &str) -> Option<String> {
    // Simple title extraction - look for <title> tags
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
        .map(|line| {
            // Limit title length
            if line.len() > 100 {
                format!("{}...", &line[..97])
            } else {
                line
            }
        })
}

fn detect_language(content: &str) -> Option<String> {
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

fn extract_links_from_content(content: &str) -> Vec<String> {
    // Simple link extraction - find href attributes
    let mut links = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == 'h'
            && content
                .get(chars.as_str().len()..)
                .unwrap_or("")
                .starts_with("ref=\"")
        {
            // Skip to the quote
            for _ in 0..5 {
                chars.next();
            }

            let mut link = String::new();
            while let Some(ch) = chars.next() {
                if ch == '"' {
                    break;
                }
                link.push(ch);
            }

            if link.starts_with("http") && link.len() > 10 {
                links.push(link);
            }

            if links.len() >= 10 {
                break;
            } // Limit number of links extracted
        }
    }

    links
}
