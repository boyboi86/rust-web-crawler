# Rust Web Crawler - Core Library

A high-performance, feature-rich web crawling library built with modern Rust architecture. This library implements a sophisticated web crawler with advanced content processing, intelligent rate limiting, and comprehensive session management.

## 🏗️ Architecture Overview

This library follows a **feature-based architecture** organized by functionality rather than technical implementation, adhering to 4 core design principles:

1. **Modules by features/logic** with sub-modules for sub-features
2. **Building blocks assembled** into unified modules
3. **One level down organization** (feature/sub-feature)
4. **Logical cohesion** over technical separation

```
src/
├── 🚀 bin/              # Entry Points & Executables
├── ⚙️ config/           # Configuration Management
├── 🎯 core/             # Core Types & Utilities
├── 🕷️ crawler/          # Main Crawling Engine
├── 📊 logging/          # Unified Logging System
├── 🌐 network/          # Network Layer Components
├── 🔄 processing/       # Content Processing & Analysis
├── 📋 queue/            # Task Queue Management
├── 🎪 session/          # Session Management & Orchestration
├── 💾 storage/          # Data Persistence
└── 📚 lib.rs            # Library Root & Re-exports
```

## 📁 Detailed Module Structure

### 🚀 **Binary Targets** (`bin/`)

**Purpose**: Executable entry points for the crawler application.

```
bin/
└── main.rs             # Primary CLI application entry point
```

**Features**:

- Command-line interface for standalone crawling
- Configuration file support
- Interactive session management

### ⚙️ **Configuration** (`config/`)

**Purpose**: Centralized configuration management with environment-specific settings.

```
config/
├── mod.rs              # Configuration module orchestration
├── crawler.rs          # Core crawler configuration types
├── environment.rs      # Environment-specific settings
└── presets.rs          # Predefined configuration presets
```

**Key Components**:

- `WebCrawlerConfig` - Main crawler configuration
- `EnvironmentConfig` - Environment variables and settings
- `LatinWordFilter` - Language filtering configuration
- Configuration presets for common use cases

### 🎯 **Core** (`core/`)

**Purpose**: Fundamental types, traits, and utilities used throughout the library.

```
core/
├── mod.rs              # Core module orchestration
├── error.rs            # Error types and handling
├── traits.rs           # Core traits and interfaces
├── types.rs            # Fundamental data types
└── utils.rs            # Utility functions and helpers
```

**Key Types**:

- `CrawlTask` - Individual crawl task representation
- `CrawlResult` - Task execution results
- `CrawlError` - Comprehensive error handling
- `TaskStatus` & `TaskPriority` - Task management types

### 🕷️ **Crawler** (`crawler/`)

**Purpose**: Main crawling engine that orchestrates the entire crawling process.

```
crawler/
├── mod.rs              # Crawler module orchestration
└── engine.rs           # Core crawling engine implementation
```

**Features**:

- `WebCrawler` - Main crawler implementation
- Asynchronous crawling with concurrent task processing
- Intelligent depth management and URL discovery
- Integration with all other subsystems

### 📊 **Logging** (`logging/`)

**Purpose**: Unified logging system with structured events and configurable output.

```
logging/
├── mod.rs              # Logging module orchestration
├── config.rs           # Logging configuration types
├── events.rs           # Structured event definitions
└── formatter.rs        # Custom log formatters
```

**Features**:

- Structured logging with tracing support
- Event-based logging for better observability
- Configurable log levels and formats
- Performance and error tracking

### 🌐 **Network** (`network/`)

**Purpose**: Network layer handling HTTP requests, DNS resolution, and rate limiting.

```
network/
├── mod.rs              # Network module orchestration
├── client.rs           # HTTP client management
├── dns.rs              # DNS resolution and caching
├── rate_limit.rs       # Intelligent rate limiting
└── robots.rs           # robots.txt compliance
```

**Features**:

- `HttpClientManager` - Connection pooling and management
- `DnsResolver` - Cached DNS resolution
- `RateLimiter` - Domain-specific rate limiting
- `RobotsChecker` - robots.txt parsing and compliance

### 🔄 **Processing** (`processing/`)

**Purpose**: Advanced content processing and analysis with multiple specialized sub-modules.

```
processing/
├── mod.rs              # Processing module orchestration
├── content.rs          # Core content processing
├── discovery.rs        # Link discovery and extraction
├── language.rs         # Language detection and filtering
├── cleaning/           # Content cleaning and sanitization
│   ├── mod.rs          # Cleaning module orchestration
│   ├── cleaner.rs      # Main cleaning implementation
│   ├── config.rs       # Cleaning configuration
│   └── rules.rs        # Cleaning rules and patterns
├── extensive/          # Extensive link processing
│   ├── mod.rs          # Extensive processing orchestration
│   ├── config.rs       # Extensive processing configuration
│   ├── link_processor.rs # Advanced link processing
│   └── queue_manager.rs # Processing queue management
└── keyword/            # Keyword extraction and matching
    ├── mod.rs          # Keyword module orchestration
    ├── config.rs       # Keyword processing configuration
    ├── extractor.rs    # Keyword extraction algorithms
    └── matcher.rs      # Keyword matching and scoring
```

**Sub-Module Features**:

#### **Cleaning** (`processing/cleaning/`)

- HTML sanitization and content cleaning
- Configurable cleaning rules and patterns
- Text normalization and formatting

#### **Extensive** (`processing/extensive/`)

- Advanced link discovery algorithms
- Intelligent crawl queue management
- Link prioritization and filtering

#### **Keyword** (`processing/keyword/`)

- Sophisticated keyword extraction
- Relevance scoring and matching
- Configurable keyword filtering strategies

### 📋 **Queue** (`queue/`)

**Purpose**: Sophisticated task queue management with caching and priority handling.

```
queue/
├── mod.rs              # Queue module orchestration
├── cache.rs            # Queue caching mechanisms
└── task_queue.rs       # Priority task queue implementation
```

**Features**:

- Priority-based task scheduling
- Duplicate URL detection and filtering
- Queue persistence and recovery
- Performance optimized caching

### 🎪 **Session** (`session/`)

**Purpose**: High-level session management and result orchestration.

```
session/
├── mod.rs              # Session module orchestration
├── manager.rs          # Session lifecycle management
├── result_collector.rs # Result aggregation and collection
└── statistics.rs       # Session statistics and metrics
```

**Features**:

- `SessionManager` - Complete session lifecycle management
- Real-time statistics and progress tracking
- Result aggregation and storage coordination
- Session persistence and recovery

### 💾 **Storage** (`storage/`)

**Purpose**: Data persistence and metrics storage with multiple output formats.

```
storage/
├── mod.rs              # Storage module orchestration
├── data.rs             # Data persistence implementation
└── metrics.rs          # Metrics and analytics storage
```

**Features**:

- Multiple storage backends (JSON, CSV, custom)
- Real-time metrics collection and storage
- Configurable data retention policies
- Export and import functionality

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.70+ (Edition 2024)
- **Tokio**: Async runtime for concurrent operations

### Installation

```bash
# Add to your Cargo.toml
[dependencies]
rust_web_crawler = { path = "path/to/rust_web_crawler" }

# Or clone and build
git clone <repository>
cd rust_web_crawler
cargo build --release
```

### Quick Start

```rust
use rust_web_crawler::{WebCrawler, WebCrawlerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = WebCrawlerConfig::default();

    // Initialize crawler
    let crawler = WebCrawler::new(config, 10, 3)?;

    // Start crawling
    let url = url::Url::parse("https://example.com")?;
    let result = crawler.init_crawling(url).await?;

    println!("Crawled content: {:?}", result);
    Ok(())
}
```

### CLI Usage

```bash
# Run the CLI application
cargo run --bin main

# With configuration file
cargo run --bin main -- --config config.toml

# Help information
cargo run --bin main -- --help
```

## 📦 Dependencies

### Core Dependencies

- **tokio 1.0** - Async runtime with full features
- **reqwest 0.11** - HTTP client with JSON and SOCKS support
- **scraper 0.13** - HTML parsing and CSS selector support
- **anyhow 1.0** - Error handling and context

### Content Processing

- **lol_html 1.2** - High-performance HTML parsing
- **whatlang 0.16** - Language detection
- **unicode-segmentation 1.10** - Text segmentation
- **regex 1.10** - Pattern matching

### Utilities

- **uuid 1.0** - Unique identifier generation
- **serde 1.0** - Serialization framework
- **url 2.4** - URL parsing and manipulation
- **bloom 0.3** - Bloom filter for duplicate detection

### Development

- **criterion 0.5** - Benchmarking framework
- **tempfile 3.8** - Temporary file management for testing

## 🎯 Key Features

### ✅ **High Performance**

- Asynchronous concurrent crawling
- Intelligent connection pooling
- Memory-efficient processing
- Bloom filter duplicate detection

### ✅ **Advanced Content Processing**

- Multi-language content analysis
- Sophisticated keyword extraction
- Configurable content cleaning
- Link discovery algorithms

### ✅ **Intelligent Rate Limiting**

- Domain-specific rate limiting
- Adaptive delay algorithms
- robots.txt compliance
- Respectful crawling practices

### ✅ **Robust Error Handling**

- Comprehensive error types
- Automatic retry mechanisms
- Graceful failure handling
- Detailed error reporting

### ✅ **Flexible Configuration**

- Environment-based configuration
- Preset configurations for common use cases
- Runtime configuration updates
- Validation and error checking

### ✅ **Comprehensive Monitoring**

- Real-time progress tracking
- Detailed session statistics
- Performance metrics collection
- Structured logging and events

## 🔧 Configuration

### Basic Configuration

```rust
use rust_web_crawler::WebCrawlerConfig;

let config = WebCrawlerConfig {
    max_total_urls: 1000,
    max_crawl_depth: 3,
    enable_extension_crawling: true,
    enable_keyword_filtering: true,
    target_words: vec!["rust".to_string(), "web".to_string()],
    ..WebCrawlerConfig::default()
};
```

### Environment Configuration

```bash
export RUST_LOG=info
export CRAWLER_MAX_DEPTH=5
export CRAWLER_RATE_LIMIT=1000
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test core::

# Run benchmarks
cargo bench
```

## 📈 Performance

### Benchmarks

The library includes comprehensive benchmarks for:

- Core crawling operations
- Content processing algorithms
- Queue operations
- Network performance

Run benchmarks with:

```bash
cargo bench
```

### Performance Characteristics

- **Throughput**: 1000+ pages/minute (depending on configuration)
- **Memory**: Efficient memory usage with streaming processing
- **Concurrency**: Configurable concurrent request limits
- **Caching**: Intelligent caching for DNS and duplicate detection

## 🛠️ Contributing

1. Follow the feature-based architecture pattern
2. Ensure comprehensive error handling
3. Add appropriate tests for new functionality
4. Update documentation for new features
5. Run benchmarks for performance-critical changes

## 📚 Documentation

- **API Documentation**: `cargo doc --open`
- **Examples**: See `examples/` directory
- **Architecture Guide**: This README and module documentation
- **Performance Guide**: Benchmark results and optimization tips

---

**Built with modern Rust architecture for high-performance web crawling and content analysis.**
