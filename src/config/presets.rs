use crate::config::{LatinWordFilter, WebCrawlerConfig};
use crate::core::types::RateConfig;
use crate::core::{DomainRateLimit, LangType, RetryConfig};
use crate::session::CrawlSessionConfig;
/// Configuration presets and factories for common crawling scenarios
///
/// This module provides pre-configured setups for different types of crawling operations,
/// reducing the need to manually configure all options in main.rs files.
use std::collections::HashMap;

/// Create a production-ready configuration with conservative rate limits
pub fn create_production_config() -> WebCrawlerConfig {
    let mut domain_rate_limits = HashMap::new();

    // Conservative rate limits for news sites
    domain_rate_limits.insert(
        "bbc.com".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 2,
                window_size_ms: 1000,
            },
        },
    );
    domain_rate_limits.insert(
        "news.naver.com".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 1,
                window_size_ms: 2000,
            },
        },
    );
    domain_rate_limits.insert(
        "spiegel.de".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 1,
                window_size_ms: 2000,
            },
        },
    );

    // Faster for test sites
    domain_rate_limits.insert(
        "httpbin.org".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 5,
                window_size_ms: 1000,
            },
        },
    );
    domain_rate_limits.insert(
        "example.com".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 3,
                window_size_ms: 1000,
            },
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
        min_word_length: 100,
        user_agent: "Rust-Web-Crawler/1.0 (Educational Purpose)".to_string(),
        default_rate_limit: Some(DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 3,
                window_size_ms: 1000,
            },
        }),
        domain_rate_limits: Some(domain_rate_limits),
        retry_config: Some(RetryConfig {
            max_retries: 3,
            timing: crate::core::types::TimingConfig {
                base_delay_ms: 1000,
                max_delay_ms: 10000,
                backoff_multiplier: 2.0,
            },
            jitter_factor: 0.2,
        }),
        proxy_pool: vec![],
        avoid_url_extensions: vec![],
        logging_config: None,
        enable_extension_crawling: false,
        max_crawl_depth: 2,
        max_total_urls: 100,
        enable_keyword_filtering: false,
        latin_word_filter: create_enhanced_latin_filter(),
    }
}

/// Create a development-friendly configuration with relaxed rate limits
pub fn create_development_config() -> WebCrawlerConfig {
    let mut domain_rate_limits = HashMap::new();

    // Relaxed rate limits for development
    domain_rate_limits.insert(
        "httpbin.org".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 10,
                window_size_ms: 1000,
            },
        },
    );
    domain_rate_limits.insert(
        "example.com".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 5,
                window_size_ms: 1000,
            },
        },
    );

    WebCrawlerConfig {
        base_url: vec![
            "https://httpbin.org".to_string(),
            "https://example.com".to_string(),
        ],
        accepted_languages: vec![LangType::Eng],
        target_words: vec!["test".to_string(), "content".to_string()],
        min_word_length: 50, // Lower threshold for testing
        user_agent: "Rust-Web-Crawler/1.0 (Development)".to_string(),
        default_rate_limit: Some(DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 5,
                window_size_ms: 1000,
            },
        }),
        domain_rate_limits: Some(domain_rate_limits),
        retry_config: Some(RetryConfig {
            max_retries: 2,
            timing: crate::core::types::TimingConfig {
                base_delay_ms: 500,
                max_delay_ms: 3000,
                backoff_multiplier: 1.5,
            },
            jitter_factor: 0.1,
        }),
        proxy_pool: vec![],
        avoid_url_extensions: vec![],
        logging_config: None,
        enable_extension_crawling: true,
        max_crawl_depth: 1,
        max_total_urls: 20,
        enable_keyword_filtering: false,
        latin_word_filter: create_basic_latin_filter(),
    }
}

/// Create a demo configuration optimized for demonstrations
pub fn create_demo_config() -> WebCrawlerConfig {
    let mut domain_rate_limits = HashMap::new();

    // Demo-friendly rate limits
    domain_rate_limits.insert(
        "httpbin.org".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 5,
                window_size_ms: 1000,
            },
        },
    );
    domain_rate_limits.insert(
        "example.com".to_string(),
        DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 2,
                window_size_ms: 1000,
            },
        },
    );

    WebCrawlerConfig {
        base_url: vec![
            "https://httpbin.org".to_string(),
            "https://example.com".to_string(),
        ],
        accepted_languages: vec![LangType::Eng, LangType::Fra, LangType::Deu],
        target_words: vec![
            "demo".to_string(),
            "example".to_string(),
            "test".to_string(),
        ],
        min_word_length: 30,
        user_agent: "Rust-Web-Crawler/1.0 (Demo)".to_string(),
        default_rate_limit: Some(DomainRateLimit {
            rate: RateConfig {
                max_requests_per_second: 3,
                window_size_ms: 1000,
            },
        }),
        domain_rate_limits: Some(domain_rate_limits),
        retry_config: Some(RetryConfig {
            max_retries: 2,
            timing: crate::core::types::TimingConfig {
                base_delay_ms: 500,
                max_delay_ms: 5000,
                backoff_multiplier: 2.0,
            },
            jitter_factor: 0.2,
        }),
        proxy_pool: vec![],
        avoid_url_extensions: vec![],
        logging_config: None,
        enable_extension_crawling: false,
        max_crawl_depth: 1,
        max_total_urls: 10,
        enable_keyword_filtering: true,
        latin_word_filter: create_basic_latin_filter(),
    }
}

/// Create session config for production crawling
pub fn create_production_session_config() -> CrawlSessionConfig {
    CrawlSessionConfig {
        crawler_config: create_production_config(),
        max_concurrent_requests: 5,
        max_depth: 3,
        max_retries: 3,
        session_timeout: Some(std::time::Duration::from_secs(600)), // 10 minutes
        enable_storage: true,
        storage_path: Some("./crawl_data".to_string()),
    }
}

/// Create session config for development
pub fn create_development_session_config() -> CrawlSessionConfig {
    CrawlSessionConfig {
        crawler_config: create_development_config(),
        max_concurrent_requests: 3,
        max_depth: 2,
        max_retries: 2,
        session_timeout: Some(std::time::Duration::from_secs(300)), // 5 minutes
        enable_storage: true,
        storage_path: Some("./dev_crawl_data".to_string()),
    }
}

/// Create session config for demos
pub fn create_demo_session_config() -> CrawlSessionConfig {
    CrawlSessionConfig {
        crawler_config: create_demo_config(),
        max_concurrent_requests: 2,
        max_depth: 1,
        max_retries: 1,
        session_timeout: Some(std::time::Duration::from_secs(120)), // 2 minutes
        enable_storage: true,
        storage_path: Some("./demo_crawl_data".to_string()),
    }
}

/// Create enhanced Latin word filter with comprehensive exclusions
fn create_enhanced_latin_filter() -> LatinWordFilter {
    LatinWordFilter {
        exclude_numeric: true,
        excluded_words: vec![
            // Common English stop words
            "the".to_string(),
            "and".to_string(),
            "or".to_string(),
            "but".to_string(),
            "in".to_string(),
            "on".to_string(),
            "at".to_string(),
            "to".to_string(),
            "for".to_string(),
            "of".to_string(),
            "with".to_string(),
            "by".to_string(),
            "is".to_string(),
            "are".to_string(),
            "was".to_string(),
            "were".to_string(),
            "be".to_string(),
            "been".to_string(),
            "have".to_string(),
            "has".to_string(),
            "had".to_string(),
            "this".to_string(),
            "that".to_string(),
            "these".to_string(),
            "those".to_string(),
            "a".to_string(),
            "an".to_string(),
            "as".to_string(),
            "if".to_string(),
            "then".to_string(),
            "than".to_string(),
            "when".to_string(),
            "where".to_string(),
            "why".to_string(),
            "how".to_string(),
            "what".to_string(),
            "who".to_string(),
            "which".to_string(),
            "will".to_string(),
            "would".to_string(),
            "can".to_string(),
            "could".to_string(),
            "should".to_string(),
            "may".to_string(),
            "might".to_string(),
            "must".to_string(),
            "shall".to_string(),
        ],
        min_word_length: 4,
    }
}

/// Create basic Latin word filter for simple filtering
fn create_basic_latin_filter() -> LatinWordFilter {
    LatinWordFilter {
        exclude_numeric: true,
        excluded_words: vec![
            "the".to_string(),
            "and".to_string(),
            "or".to_string(),
            "but".to_string(),
            "in".to_string(),
            "on".to_string(),
            "at".to_string(),
            "to".to_string(),
            "for".to_string(),
            "of".to_string(),
            "with".to_string(),
            "by".to_string(),
            "is".to_string(),
            "are".to_string(),
            "was".to_string(),
            "were".to_string(),
        ],
        min_word_length: 3,
    }
}
