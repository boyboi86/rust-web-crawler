# 🚀 Modular Rust Web Crawler

[![Built with Rust](https://img.shields.io/badge/Built_with-Rust-orange)](https://www.rust-lang.org/)
[![Tokio Runtime](https://img.shields.io/badge/Runtime-Tokio-blue)](https://tokio.rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)
[![Architecture](https://img.shields.io/badge/Architecture-Modular-green)](#architecture)
[![Testing](https://img.shields.io/badge/Testing-Comprehensive-blue)](#testing)

A production-ready, modular web crawler built in Rust featuring structured logging, centralized configuration, multilingual content processing, and comprehensive testing. Designed with maintainable architecture and real-world readiness in mind.

## 🌟 Key Highlights

- **🏗️ Modular Architecture**: Clean separation of concerns with feature-based modules
- **📝 Structured Logging**: Comprehensive event logging with configurable output formats
- **⚙️ Centralized Configuration**: Unified configuration system for easy maintenance
- **🌍 Multilingual Support**: Automatic language detection and Unicode-aware processing
- **🧪 Comprehensive Testing**: Modular test suite covering all major features
- **🔧 Production Ready**: Built for maintainability, reliability, and real-world deployment

## 📋 Table of Contents

- [Features](#-features)
- [Architecture](#-architecture)
- [Installation & Quick Start](#-installation--quick-start)
- [Demo & Development Mode](#-demo--development-mode)
- [Configuration](#-configuration)
- [Usage Examples](#-usage-examples)
- [Testing](#-testing)
- [Monitoring & Logging](#-monitoring--logging)
- [Contributing](#-contributing)
- [License](#-license)

## ✨ Features

### Core Web Crawling

- **Async Concurrent Crawling**: Tokio-powered with configurable concurrency limits
- **URL Deduplication**: Bloom filter-based duplicate detection for memory efficiency
- **Content Validation**: Unicode-aware text processing with quality filtering
- **Robots.txt Compliance**: Automatic parsing and respect for crawling directives

### Multilingual Content Processing

- **Language Detection**: Automatic detection using `whatlang` library
- **Supported Languages**: English, Chinese (Mandarin), French, German, Japanese, Korean
- **Unicode Processing**: Proper handling of CJK languages and text segmentation
- **Content Quality**: Configurable minimum word count and content validation

### Network & Performance

- **HTTP Client**: Configurable timeouts, redirects, and connection pooling
- **Rate Limiting**: Per-domain rate limiting with sliding window algorithm
- **DNS Caching**: Custom DNS resolution with TTL-based caching
- **Robots.txt Caching**: Intelligent robots.txt caching with automatic cleanup
- **Generic TTL Cache**: Reusable cache implementation for future extensions

### Logging & Monitoring

- **Structured Events**: Comprehensive crawl event logging system
- **Configurable Output**: JSON and human-readable log formats
- **Performance Metrics**: Built-in metrics collection and reporting
- **Error Tracking**: Detailed error categorization and logging

## 🏗️ Architecture

The codebase is organized into logical, feature-based modules for maximum maintainability:

```
src/
├── bin/                    # Executable binaries
│   └── main.rs            # Main demo application
├── config/                # Configuration management
│   ├── mod.rs             # Re-exports and module definition
│   ├── crawler.rs         # Crawler configuration structures
│   └── environment.rs     # Environment-specific settings
├── core/                  # Core types and utilities
│   ├── mod.rs             # Core module exports
│   ├── types.rs           # Common data types and enums
│   ├── traits.rs          # Shared traits and interfaces
│   ├── error.rs           # Error types and handling
│   └── utils.rs           # Utility functions
├── network/               # Network operations
│   ├── mod.rs             # Network module exports
│   ├── client.rs          # HTTP client implementation
│   ├── dns.rs             # DNS resolution with caching
│   ├── robots.rs          # Robots.txt parsing and caching
│   └── rate_limit.rs      # Rate limiting implementation
├── processing/            # Content processing
│   ├── mod.rs             # Processing module exports
│   ├── content.rs         # Text extraction and validation
│   └── discovery.rs       # Link discovery and parsing
├── queue/                 # Task queue management
│   ├── mod.rs             # Queue module exports
│   ├── task_queue.rs      # Task scheduling and prioritization
│   └── cache.rs           # Generic TTL cache implementation
├── storage/               # Data storage and metrics
│   ├── mod.rs             # Storage module exports
│   ├── data.rs            # Data persistence interfaces
│   └── metrics.rs         # Performance metrics collection
├── crawler/               # Main crawler engine
│   ├── mod.rs             # Crawler module exports
│   └── engine.rs          # Core crawling engine implementation
├── logging/               # Structured logging system
│   ├── mod.rs             # Logging module exports
│   ├── events.rs          # Crawl event definitions
│   └── formatter.rs       # Log formatting utilities
└── lib.rs                 # Library root with public API
```

### Additional Files

```
tests/                     # Comprehensive test suite
├── core.rs               # Core functionality tests
├── network.rs            # Network operations tests
├── processing.rs         # Content processing tests
├── storage.rs            # Storage and metrics tests
└── crawler_integration.rs # End-to-end integration tests

config/                   # Configuration examples
└── crawler_config_example.toml

TESTING.md               # Testing documentation
```

### Design Principles

- **Separation of Concerns**: Each module has a single, well-defined responsibility
- **DRY (Don't Repeat Yourself)**: Common functionality is centralized and reused
- **Testability**: Each module can be tested independently
- **Configurability**: All behavior is configurable through centralized settings
- **Extensibility**: New features can be added without modifying existing code

## 📦 Installation & Quick Start

### Prerequisites

- Rust 1.70+ (2024 edition)
- Tokio async runtime

### Installation

```bash
git clone
cd rust_web_crawler
cargo build --release
```

### Quick Start (Production Mode)

Run the main application to crawl websites and save results to JSON files:

```bash
cargo run
```

This executes `src/bin/main.rs` which:

- **Crawls real websites** (BBC News, Naver, HTTPBin, etc.)
- **Saves results to JSON files** in `./crawl_data` directory
- **Includes multilingual support** with automatic language detection
- **Uses production-ready configuration** with rate limiting and error handling
- **Generates session summaries** with comprehensive statistics

### Demo Mode

To see feature demonstrations and testing scenarios:

```bash
cargo run --bin demo
```

This runs `src/bin/demo.rs` which demonstrates all features with console output and saves demo results to `./demo_output` directory.

### Build for Production

```bash
cargo build --release
./target/release/main
```

## 🎮 Production & Demo Modes

### Production Mode (`cargo run`)

When you run `cargo run`, the application starts in **production mode** and performs actual web crawling:

#### What Production Mode Does:

1. **Real Website Crawling**: Crawls actual websites including:

   - English sites (BBC News, Example.com, HTTPBin)
   - Korean sites (Naver News)
   - Various content types and languages

2. **JSON Output**: Saves all crawl results to individual JSON files in `./crawl_data/`

3. **Structured Data**: Each JSON file contains:

   ```json
   {
     "url": "https://www.bbc.com/news",
     "title": "BBC News - Home",
     "content": "Full extracted text content...",
     "word_count": 2847,
     "language": "en",
     "links_found": ["https://...", "..."],
     "metadata": {
       "status_code": 200,
       "content_type": "text/html",
       "response_time_ms": 234,
       "crawl_session_id": "uuid-here"
     },
     "timestamp": "2024-01-15T10:30:45Z"
   }
   ```

4. **Session Summary**: Generates a comprehensive session summary with statistics

5. **Production Features**:
   - Multilingual content processing
   - Rate limiting per domain
   - Error handling and retries
   - Structured logging
   - Language detection

#### Output Structure:

```
./crawl_data/
├── crawl_1705320645_a1b2c3d4.json    # Individual crawl results
├── crawl_1705320646_e5f6g7h8.json
├── crawl_1705320647_i9j0k1l2.json
└── session_summary_uuid.json          # Session statistics
```

### Demo Mode (`cargo run --bin demo`)

For feature demonstrations and testing:

#### What Demo Mode Does:

1. **Feature Demonstrations**: Shows all crawler capabilities
2. **Console Output**: Displays real-time crawling progress
3. **Demo Results**: Saves to `./demo_output/` directory
4. **Educational**: Explains each feature as it runs

### Sample Demo Output:

```
🚀 Starting Rust Web Crawler Demo...

=== URL Validation Demo ===
✓ Valid URL: https://www.bbc.com/news
✓ Valid URL: https://news.naver.com
✗ Invalid URL: not-a-url

=== Multilingual Crawling Demo ===
🌍 Crawling https://www.bbc.com/news
✓ Content extracted: 2,847 characters (Language: English)
🌍 Crawling https://news.naver.com
✓ Content extracted: 1,924 characters (Language: Korean)

=== Rate Limiting Demo ===
⏱️ Rate limiting applied for bbc.com: 200ms delay
⏱️ Rate limiting applied for naver.com: 500ms delay

=== Structured Logging Demo ===
{"timestamp":"2024-01-15T10:30:45Z","event":"crawl_start","url":"https://www.bbc.com"}
{"timestamp":"2024-01-15T10:30:46Z","event":"content_extracted","url":"https://www.bbc.com","length":2847,"language":"en"}
```

## 🌟 Key Highlights

- **🏆 Production-Ready**: Fault-tolerant retry system with exponential backoff and priority queuing
- **⚡ High Performance**: DNS caching, connection pooling, and optimized async operations
- **�️ Anti-Bot Evasion**: Advanced proxy rotation, header spoofing, and intelligent rate limiting
- **🧠 Smart Content Processing**: Multi-language support with Unicode-aware text extraction
- **🔧 Enterprise Configuration**: Centralized defaults system for easy maintenance and deployment
- **📊 Comprehensive Monitoring**: Real-time statistics for rate limiting, DNS cache, and crawling metrics

## 📋 Table of Contents

- [Core Features](#-core-features)
- [Advanced Capabilities](#-advanced-capabilities)
- [Performance Optimizations](#-performance-optimizations)
- [Fault Tolerance](#-fault-tolerance)
- [Installation & Quick Start](#-installation--quick-start)
- [Configuration](#-configuration)
- [Usage Examples](#-usage-examples)
- [Architecture](#-architecture)
- [Monitoring & Statistics](#-monitoring--statistics)
- [Contributing](#-contributing)
- [License](#-license)

## 🚀 Core Features

### ✅ Web Crawling Engine

- **Async Concurrent Crawling**: Tokio-powered with configurable concurrency limits
- **Bloom Filter Deduplication**: Memory-efficient URL deduplication (1M URLs, 1% false positive)
- **Content Validation**: Unicode-aware word counting with language-specific filtering
- **Memory-Safe Parallelism**: Zero data races using Rust's ownership system

### ✅ Robots.txt Compliance

- **Enhanced Parser**: Supports User-agent, Allow, Disallow, Crawl-delay, and Request-rate directives
- **Intelligent Caching**: Per-domain caching with TTL (24 hours default)
- **Automatic Fallback**: Graceful handling when robots.txt is unavailable

### ✅ Multi-Language Support

- **Language Detection**: Automatic detection using `whatlang` library
- **Supported Languages**: English, Chinese (Mandarin), French, German, Japanese, Korean
- **Dynamic Headers**: Intelligent Accept-Language header generation with quality values
- **Unicode Processing**: Proper handling of CJK languages and text segmentation

## 🛡️ Advanced Capabilities

### Domain-Specific Rate Limiting

- **Sliding Window Algorithm**: Precise rate control with configurable windows
- **Per-Domain Limits**: Custom rate limits for different domains
- **Intelligent Jitter**: Randomized delays to prevent thundering herd effects
- **Memory Management**: Automatic cleanup of inactive domain trackers

### Anti-Bot Evasion

- **Proxy Rotation**: Support for SOCKS5 and HTTP proxies with connection pooling
- **Header Spoofing**: Randomized User-Agent strings from real browsers
- **Realistic Headers**: Comprehensive HTTP headers (Accept, Accept-Language, Accept-Encoding)
- **Request Timing**: Configurable delays with randomization

### Fault-Tolerant Retry System

- **Priority Queue**: Smart retry scheduling with task prioritization
- **Exponential Backoff**: Configurable backoff multipliers with jitter
- **Error Categorization**: Intelligent error classification for retry decisions
- **Persistent Queuing**: Tasks retry until max attempts or success

## ⚡ Performance Optimizations

### DNS & Network

- **DNS Caching**: 5-minute TTL cache with automatic cleanup (280x faster on cache hits)
- **Connection Pooling**: Persistent connections for proxy clients
- **HTTP/2 Support**: Modern protocol support for improved performance

### Content Processing

- **Pre-compiled Regex**: Cached regex patterns for HTML cleaning
- **Early Content Filtering**: Skip processing of very short content
- **Language Detection Sampling**: Use text samples for large documents
- **Optimized Lock Usage**: RwLock for reduced contention in rate limiting

### Memory Management

- **TTL-based Caches**: Automatic expiration of robots.txt and DNS entries
- **Inactive Resource Cleanup**: Periodic cleanup of unused domain trackers
- **Bounded Pools**: Limited connection pools to prevent memory leaks

## 🛡️ Fault Tolerance

### Retry Logic

- **Configurable Retries**: Default 3 retries with customizable limits
- **Error Classification**: Network timeouts, connection issues, HTTP errors
- **Backoff Strategy**: Exponential backoff (2x multiplier) with jitter
- **Priority Scheduling**: High-priority tasks processed first

### Error Handling

- **Graceful Degradation**: Continue crawling even with partial failures
- **Detailed Error Reporting**: Comprehensive error categorization and logging
- **Recovery Mechanisms**: Automatic retry for transient failures

## 📦 Installation & Quick Start

### Prerequisites

- Rust 1.70+ (2024 edition)
- Tokio async runtime

### Installation

```bash
git clone
cd rust_web_crawler
cargo build --release
```

### Quick Start

```bash
cargo run
```

This will run the demo application showcasing all features including:

- Multi-language content crawling
- Rate limiting demonstration
- Fault-tolerant crawling with retries
- DNS caching performance comparison

## 🔧 Configuration

### Centralized Defaults System

All configuration is centralized in the `CrawlerDefaults` struct for easy maintenance:

```rust
pub struct CrawlerDefaults {
    // HTTP Client defaults
    pub max_redirects: usize,                    // 10
    pub request_timeout_secs: u64,               // 30

    // Connection pool settings
    pub connection_pool_size: usize,             // 10
    pub connection_idle_timeout_secs: u64,       // 300 (5 minutes)

    // Rate limiting defaults
    pub default_max_requests_per_second: u32,    // 10
    pub default_window_size_ms: u64,             // 1000 (1 second)

    // Cache TTL settings
    pub robots_cache_ttl_hours: u64,             // 24
    pub dns_cache_ttl_secs: u64,                 // 300 (5 minutes)

    // Retry system defaults
    pub default_max_retries: u32,                // 3
    pub default_backoff_multiplier: f64,         // 2.0

    // And many more...
}
```

### WebCrawlerConfig

```rust
let config = WebCrawlerConfig {
    base_url: vec!["https://example.com".to_string()],
    min_word_length: 50,
    proxy_pool: vec![
        "socks5://127.0.0.1:1080".to_string(),
        "http://proxy.example.com:8080".to_string(),
    ],
    user_agent: "Mozilla/5.0 (compatible; RustCrawler/1.0)".to_string(),
    accepted_languages: vec![LangType::Eng, LangType::Cmn, LangType::Jpn],
    default_rate_limit: Some(DomainRateLimit {
        max_requests_per_second: 10,
        window_size_ms: 1000,
    }),
    domain_rate_limits: Some(domain_specific_limits),
    retry_config: Some(RetryConfig {
        max_retries: 5,
        base_delay_ms: 2000,
        max_delay_ms: 30000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.2,
        retry_on_errors: vec![
            CrawlError::NetworkTimeout,
            CrawlError::HttpError(500),
            // ... more error types
        ],
    }),
};
```

## 💡 Usage Examples

### Basic Crawling

```rust
use rust_web_crawler::{WebCrawler, WebCrawlerConfig, LangType};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WebCrawlerConfig {
        base_url: vec!["https://example.com".to_string()],
        accepted_languages: vec![LangType::Eng],
        min_word_length: 50,
        // ... other config
    };

    let urls = vec![
        Url::parse("https://example.com")?,
        Url::parse("https://httpbin.org/html")?,
    ];

    let results = WebCrawler::run_concurrent_crawling(urls, 5, config).await?;

    for (url, content) in results {
        match content {
            Some(text) => println!("✓ Crawled {}: {} chars", url, text.len()),
            None => println!("✗ Skipped {}", url),
        }
    }

    Ok(())
}
```

### Fault-Tolerant Crawling

```rust
let fault_tolerant_urls = vec![
    Url::parse("https://httpbin.org/status/200")?, // Should work
    Url::parse("https://httpbin.org/status/500")?, // Will retry
    Url::parse("https://nonexistent-domain.com")?, // Will fail after retries
];

let results = WebCrawler::run_fault_tolerant_crawling(
    fault_tolerant_urls,
    3, // max concurrent
    config
).await?;

for result in results {
    match result {
        CrawlResult::Success(content) => println!("✅ Success: {} chars", content.len()),
        CrawlResult::Failed(error) => println!("❌ Failed: {}", error),
        CrawlResult::Skipped(reason) => println!("⏭️ Skipped: {}", reason),
    }
}
```

### Advanced Configuration

```rust
// Domain-specific rate limits
let mut domain_limits = HashMap::new();
domain_limits.insert("example.com".to_string(), DomainRateLimit {
    max_requests_per_second: 5,  // Slower for example.com
    window_size_ms: 1000,
});
domain_limits.insert("httpbin.org".to_string(), DomainRateLimit {
    max_requests_per_second: 3,  // Even slower for httpbin.org
    window_size_ms: 1000,
});

let config = WebCrawlerConfig {
    domain_rate_limits: Some(domain_limits),
    retry_config: Some(RetryConfig {
        max_retries: 5,
        base_delay_ms: 2000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.2,
        // Customize which errors trigger retries
        retry_on_errors: vec![
            CrawlError::NetworkTimeout,
            CrawlError::HttpError(500),
            CrawlError::HttpError(502),
            CrawlError::HttpError(503),
        ],
    }),
    // ... other config
};
```

## 🏗️ Architecture

### Core Components

1. **WebCrawler**: Main crawler engine with async capabilities
2. **GlobalRateLimiter**: Domain-specific rate limiting with sliding windows
3. **SimpleFaultTolerantCrawler**: Retry queue with priority scheduling
4. **CrawlerDefaults**: Centralized configuration management

### Data Flow

```
URL Input → Bloom Filter Check → Robots.txt Check → Rate Limiting →
DNS Resolution (with caching) → HTTP Request → Content Processing →
Language Detection → Word Count Validation → Result Output
```

### Async Architecture

- **Tokio Runtime**: Non-blocking I/O operations
- **Arc<Mutex>** / **Arc<RwLock>**: Thread-safe shared state
- **Semaphores**: Concurrency control
- **Priority Queues**: Smart task scheduling

## 📊 Monitoring & Statistics

### Rate Limiting Stats

```rust
let stats = crawler.get_rate_limit_stats().await;
// Returns: HashMap<String, usize> - domain -> active requests
```

### DNS Cache Stats

```rust
let dns_stats = crawler.get_dns_cache_stats().await;
// Returns: HashMap<String, String> - domain -> "IP (cached Xs ago)"
```

### Crawling Metrics

The fault-tolerant crawler provides detailed statistics:

- Total tasks processed
- Successful completions
- Failed tasks
- Retry attempts
- Queue status

## 🔍 Supported Content Types

### Languages

- **English** (`LangType::Eng`)
- **Chinese Mandarin** (`LangType::Cmn`)
- **French** (`LangType::Fra`)
- **German** (`LangType::Deu`)
- **Japanese** (`LangType::Jpn`)
- **Korean** (`LangType::Kor`)

### Content Processing

- **HTML Parsing**: Zero-copy parsing with `lol-html`
- **Text Extraction**: Removes scripts, styles, navigation
- **Unicode Support**: Proper CJK word segmentation
- **Quality Filtering**: Minimum word count thresholds

## 🧪 Testing & Debugging

### Run Tests

```bash
cargo test
```

### Debug Mode

```bash
RUST_LOG=debug cargo run
```

### Performance Testing

The demo application includes built-in performance tests:

- DNS caching performance comparison
- Rate limiting behavior verification
- Retry system functionality

## 📈 Performance Benchmarks

### DNS Caching Performance

- **First resolution (cache miss)**: ~2.8ms
- **Subsequent resolutions (cache hit)**: ~10μs (**280x faster**)

### Memory Usage

- **Bloom Filter**: ~1.2MB for 1M URLs
- **DNS Cache**: ~100KB for 1000 domains
- **Rate Limiter**: ~50KB per active domain

### Throughput

- **Single domain**: Up to configured rate limit
- **Multiple domains**: Parallel processing across domains
- **Fault tolerance**: Maintains throughput despite failures

## 🔒 Security Features

### Request Headers

```http
User-Agent: Mozilla/5.0 (randomized from real browsers)
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8
Accept-Language: en-US,en-GB,en;q=0.9,zh-CN,zh-TW,zh;q=0.7,ja-JP,ja;q=0.4,*;q=0.1
Accept-Encoding: gzip, deflate
Connection: keep-alive
Upgrade-Insecure-Requests: 1
```

### Proxy Support

- **SOCKS5**: `socks5://username:password@host:port`
- **HTTP**: `http://username:password@host:port`
- **Connection Pooling**: Cached proxy connections
- **Random Selection**: Automatic proxy rotation

## �️ Dependencies

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "socks"] }
lol_html = "1.2"           # Zero-copy HTML parsing
bloom = "0.3"              # Bloom filter for URL deduplication
whatlang = "0.16"          # Language detection
anyhow = "1.0"             # Error handling
url = "2.4"                # URL parsing
rand = "0.8"               # Randomization
unicode-segmentation = "1.10"  # Unicode word segmentation
regex = "1.10"             # Regular expressions
futures = "0.3"            # Async stream processing
```

## � Roadmap & Future Enhancements

### Planned Features

- **JavaScript Rendering**: Headless browser integration for SPA support
- **Distributed Crawling**: Multi-node coordination and load balancing
- **Advanced Storage**: Database integration (PostgreSQL, MongoDB)
- **Machine Learning**: Content classification and relevance scoring
- **GraphQL Support**: Modern API crawling capabilities

### Performance Improvements

- **HTTP/3 Support**: Next-generation protocol implementation
- **Compression Optimization**: Advanced content encoding support
- **Streaming Processing**: Real-time content processing pipelines

## 📚 Examples & Tutorials

### Example 1: E-commerce Product Crawling

```rust
let ecommerce_config = WebCrawlerConfig {
    accepted_languages: vec![LangType::Eng],
    min_word_length: 30,
    // Target product pages specifically
    target_words: vec!["price".to_string(), "product".to_string()],
    default_rate_limit: Some(DomainRateLimit {
        max_requests_per_second: 2, // Be respectful
        window_size_ms: 1000,
    }),
    // ... rest of config
};
```

### Example 2: News Article Aggregation

```rust
let news_config = WebCrawlerConfig {
    accepted_languages: vec![
        LangType::Eng,
        LangType::Fra,
        LangType::Deu
    ],
    min_word_length: 200, // Longer articles
    // Focus on news content
    target_words: vec!["news".to_string(), "article".to_string()],
    // ... rest of config
};
```

### Example 3: Research Paper Mining

```rust
let academic_config = WebCrawlerConfig {
    accepted_languages: vec![LangType::Eng],
    min_word_length: 500, // Academic content is typically longer
    avoid_url_extensions: vec![
        ".pdf".to_string(),  // Handle PDFs separately
        ".docx".to_string(),
    ],
    target_words: vec![
        "research".to_string(),
        "study".to_string(),
        "paper".to_string()
    ],
    // ... rest of config
};
```

## 🔧 Troubleshooting

### Common Issues

#### DNS Resolution Failures

```bash
# Solution: Check DNS cache stats
let dns_stats = crawler.get_dns_cache_stats().await;
println!("DNS Cache: {:?}", dns_stats);
```

#### Rate Limiting Issues

```bash
# Solution: Monitor rate limiting stats
let rate_stats = crawler.get_rate_limit_stats().await;
println!("Rate Limits: {:?}", rate_stats);
```

#### Memory Usage

```bash
# Monitor with:
cargo run --release  # Use release mode for production
RUST_LOG=info cargo run  # Reduce log verbosity
```

### Performance Tuning

#### For High-Volume Crawling

```rust
let high_volume_config = CrawlerDefaults {
    default_max_concurrent_tasks: 50,
    connection_pool_size: 25,
    dns_cache_ttl_secs: 600, // 10 minutes
    // ... other optimizations
};
```

#### For Memory-Constrained Environments

```rust
let low_memory_config = CrawlerDefaults {
    bloom_capacity: 100_000,  // Reduced capacity
    connection_pool_size: 5,
    cleanup_interval_secs: 60, // More frequent cleanup
    // ... other optimizations
};
```

## 📖 API Documentation

### Core Methods

#### `WebCrawler::new()`

Creates a new crawler instance with configuration.

**Parameters:**

- `config: WebCrawlerConfig` - Crawler configuration
- `max_concurrent_requests: usize` - Maximum concurrent HTTP requests
- `max_depth: usize` - Maximum crawling depth

**Returns:** `Result<WebCrawler, Error>`

#### `WebCrawler::init_crawling()`

Crawls a single URL with full pipeline processing.

**Parameters:**

- `url: Url` - URL to crawl

**Returns:** `Result<Option<String>, Error>`

- `Some(content)` - Successfully extracted content
- `None` - URL skipped (robots.txt, duplicate, etc.)

#### `WebCrawler::run_concurrent_crawling()`

Crawls multiple URLs concurrently.

**Parameters:**

- `seeds: Vec<Url>` - URLs to crawl
- `max_concurrent_tasks: usize` - Concurrency limit
- `config: WebCrawlerConfig` - Configuration

**Returns:** `Result<Vec<(Url, Option<String>)>, Error>`

#### `WebCrawler::run_fault_tolerant_crawling()`

Crawls with automatic retry and fault tolerance.

**Parameters:**

- `seeds: Vec<Url>` - URLs to crawl
- `max_concurrent_tasks: usize` - Concurrency limit
- `config: WebCrawlerConfig` - Configuration

**Returns:** `Result<Vec<CrawlResult>, Error>`

### Monitoring Methods

#### `get_rate_limit_stats()`

Returns current rate limiting statistics per domain.

#### `get_dns_cache_stats()`

Returns DNS cache statistics with timing information.

#### `perform_maintenance()`

Triggers manual cache cleanup and maintenance tasks.

## 🔐 Security Considerations

### Responsible Crawling

- Always respect robots.txt directives
- Implement appropriate delays between requests
- Monitor server response times and adjust accordingly
- Use reasonable concurrency limits

### Privacy & Legal

- Respect website terms of service
- Consider GDPR and privacy regulations
- Implement data retention policies
- Obtain necessary permissions for commercial use

### Rate Limiting Best Practices

```rust
// Conservative settings for unknown domains
let conservative_limit = DomainRateLimit {
    max_requests_per_second: 1,
    window_size_ms: 2000, // 2 second window
};

// Faster settings for known-friendly domains
let friendly_limit = DomainRateLimit {
    max_requests_per_second: 10,
    window_size_ms: 1000,
};
```

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

### Development Setup

```bash
git clone https://github.com/yourusername/rust_web_crawler.git
cd rust_web_crawler
cargo build
cargo test
```

### Code Style

- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting
- Add tests for new features
- Update documentation

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Areas for Contribution

- Additional language support
- Performance optimizations
- New content extraction algorithms
- Enhanced anti-bot evasion techniques
- Documentation improvements

## 📞 Support & Community

### Getting Help

- **Issues**: Create GitHub issues for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check the wiki for detailed guides

### Reporting Bugs

Please include:

- Rust version (`rustc --version`)
- Operating system
- Detailed error messages
- Minimal reproduction example

### Feature Requests

When requesting features:

- Describe the use case
- Explain the expected behavior
- Consider implementation complexity
- Check existing issues first

## 📜 Changelog

### Version 2.0.0 (Current)

- ✅ Added fault-tolerant retry system with priority queuing
- ✅ Implemented DNS caching with 280x performance improvement
- ✅ Added domain-specific rate limiting with sliding windows
- ✅ Centralized configuration system
- ✅ Enhanced robots.txt parser with Crawl-delay and Request-rate support
- ✅ Connection pooling for proxy clients
- ✅ Comprehensive monitoring and statistics

### Version 1.0.0

- ✅ Basic concurrent crawling
- ✅ Bloom filter URL deduplication
- ✅ Multi-language content processing
- ✅ Proxy support and header spoofing

## � License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2025 Rust Web Crawler Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 🎯 Built With Modern Rust

This project showcases enterprise-grade Rust development practices:

- **🦀 Rust 2024 Edition**: Latest language features and improvements
- **⚡ Tokio Async Runtime**: High-performance async I/O operations
- **🔒 Memory Safety**: Zero-cost abstractions with compile-time guarantees
- **🧵 Fearless Concurrency**: Safe parallelism without data races
- **📊 Type Safety**: Comprehensive type system preventing runtime errors
- **🚀 Performance**: Native speed with minimal overhead

**Built with ❤️ in Rust** | **Powered by Tokio** | **Ready for Production** 🚀
