# AI Agent Development Rules for Rust Web Crawler Backend

## 🤖 Core Principles for AI Agents

### ✅ **ALLOWED WITHOUT PERMISSION**

- Read any file for context understanding
- Create new `.rs` files following existing patterns
- Modify existing code for bug fixes and improvements
- Add comprehensive error handling
- Create unit tests and documentation
- Use existing configuration systems
- Run development commands: `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`

### 🔒 **REQUIRES EXPLICIT PERMISSION**

- Deleting any existing files or modules
- Major architectural changes
- Adding new dependencies to Cargo.toml
- Modifying public APIs (breaking changes)
- Security-related configurations
- Performance-critical optimizations that change behavior

## 📋 **MANDATORY CODING STANDARDS (11 Rules)**

### **1. 🚫 No Direct Hardcoding**

**Rule**: No hardcoded values directly in functions or struct fields

```rust
// ❌ BAD: Hardcoded values
fn create_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30)) // Hardcoded!
        .build()
        .unwrap()
}

// ✅ GOOD: Configuration-driven
use crate::config::defaults::DEFAULT_REQUEST_TIMEOUT_SECS;

fn create_client(timeout_secs: u64) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .unwrap()
}
```

### **2. 📦 Explicit Import Requirements**

**Rule**: Always use explicit `use` statements, never direct `crate::` or `std::` calls in code

```rust
// ❌ BAD: Direct calls in code
fn send_message() {
    let (tx, rx) = std::sync::mpsc::channel(); // Direct std:: call
    let config = crate::config::get_config(); // Direct crate:: call
}

// ✅ GOOD: Explicit imports
use std::sync::mpsc;
use crate::config;

fn send_message() {
    let (tx, rx) = mpsc::channel();
    let config = config::get_config();
}
```

### **3. 🏗️ Builder Pattern + Default Enforcement**

**Rule**: Configuration structs must implement `#[derive(Default)]` and use builder pattern

```rust
// ✅ GOOD: Proper configuration with Default
#[derive(Clone, Debug, Default)]
pub struct HttpClientConfig {
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub user_agent: String,
}

impl HttpClientConfig {
    pub fn new() -> Self {
        Self {
            timeout_secs: 30,
            max_retries: 3,
            user_agent: "RustCrawler/1.0".to_string(),
        }
    }
}

// Usage with clean fallbacks
let config = user_config.unwrap_or_default(); // Uses Default implementation
```

### **4. 🔒 Privacy-First Approach**

**Rule**: Sensitive information must be private, exposed through controlled methods

```rust
// ✅ GOOD: Private fields with controlled access
#[derive(Debug)]
pub struct ApiClient {
    client: reqwest::Client,          // Private
    api_key: String,                  // Private - sensitive
    base_url: String,                 // Private
}

impl ApiClient {
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: base_url.into(),
        }
    }

    // Controlled access - no direct exposure of sensitive data
    pub async fn make_request(&self, endpoint: &str) -> Result<String, ApiError> {
        // Implementation uses private fields safely
    }
}
```

### **5. 🧩 Composition Pattern in `common/`**

**Rule**: Small building blocks assembled in `common/` folder only

```rust
// common/http_client.rs - Building block
#[derive(Clone, Debug)]
pub struct HttpClientConfig {
    pub client: reqwest::Client,
    pub timeout: Duration,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            timeout: Duration::from_secs(30),
        }
    }
}

// common/api_params.rs - Building block
#[derive(Clone, Debug)]
pub struct ApiParams {
    api_key: String,    // Private
    endpoint: String,   // Private
}

impl ApiParams {
    pub fn new(api_key: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            endpoint: endpoint.into(),
        }
    }
}

// common/service_config.rs - Composition (ONLY in common/)
use crate::common::{HttpClientConfig, ApiParams};

#[derive(Debug)]
pub struct ServiceConfig {
    pub http: HttpClientConfig,
    pub api: ApiParams,
    pub retries: u32,
    pub enable_logging: bool,
}
```

### **6. 🏗️ Builder Pattern for External Usage**

**Rule**: Final assembled structs must use builder pattern outside `common/`

```rust
// common/service_config.rs - Builder for external use
#[derive(Default)]
pub struct ServiceConfigBuilder {
    http: Option<HttpClientConfig>,
    api: Option<ApiParams>,
    retries: Option<u32>,
    enable_logging: Option<bool>,
}

impl ServiceConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn http(mut self, config: HttpClientConfig) -> Self {
        self.http = Some(config);
        self
    }

    pub fn api(mut self, params: ApiParams) -> Self {
        self.api = Some(params);
        self
    }

    pub fn retries(mut self, count: u32) -> Self {
        self.retries = Some(count);
        self
    }

    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.enable_logging = Some(enable);
        self
    }

    pub fn build(self) -> Result<ServiceConfig, ConfigError> {
        Ok(ServiceConfig {
            http: self.http.unwrap_or_default(), // Uses Default implementation
            api: self.api.ok_or(ConfigError::MissingApiParams)?,
            retries: self.retries.unwrap_or(3),
            enable_logging: self.enable_logging.unwrap_or(true),
        })
    }
}

// Outside common/ - Only builder pattern usage allowed
use crate::common::{ServiceConfigBuilder, HttpClientConfig, ApiParams};

fn setup_service() -> Result<ServiceConfig, ConfigError> {
    let http_config = HttpClientConfig::default();
    let api_params = ApiParams::new("secret_key", "https://api.example.com");

    ServiceConfigBuilder::new()
        .http(http_config)
        .api(api_params)
        .retries(5)
        .enable_logging(true)
        .build()
}
```

## 📁 **Architecture Rules**

### **Assembly Location Rules**

- ✅ **In `common/`**: Assemble small components into larger blocks
- ✅ **In `common/`**: Create `fn new() -> Self` and traits for repeated patterns
- ✅ **Export from `common/`**: Use builder patterns for external consumption
- ❌ **Outside `common/`**: No further assembly - use components immediately

### **Import and Usage Flow**

1. **Small components** created in `common/` files grouped by purpose
2. **Assembly** happens within `common/` folder only
3. **Export** final assembled structs with builder patterns
4. **External modules** use builder pattern exclusively
5. **Immediate usage** - no re-assembly outside `common/`

## 🔍 **Pre-Implementation Review**

**IMPORTANT**: Before creating any composition parts, review ALL existing structs in `common/` folders to:

- Understand existing patterns
- Avoid duplication
- Ensure consistency
- Follow established conventions
- Identify reusable components

### **12. 🧪 Test Organization and Safety**

**Rule**: Test cases must be in separate files with specific scope, only tests can use `unwrap()`/`expect()`

```rust
// ❌ BAD: Tests mixed with production code
// src/network/client.rs
impl NetworkClient {
    pub fn new() -> Self {
        // Production code - no unwrap/expect allowed
    }
}

#[cfg(test)]
mod tests {
    // Tests in same file - discouraged for complex modules
}

// ✅ GOOD: Dedicated test files with specific scope
// tests/unit/network/test_client_creation.rs
use rust_web_crawler::network::NetworkClient;

#[test]
fn test_client_creation_with_valid_config() {
    let config = NetworkClientConfig::default();
    let client = NetworkClient::new(config).expect("Client creation should succeed"); // OK in tests
    assert!(client.is_healthy());
}

// tests/unit/network/test_client_requests.rs
#[test]
fn test_client_timeout_handling() {
    let client = create_test_client().unwrap(); // OK in tests
    // Test timeout scenarios
}

// tests/integration/test_crawler_engine.rs
#[test]
fn test_full_crawl_workflow() {
    let engine = CrawlerEngine::with_defaults().await.expect("Engine should start"); // OK in tests
    // Integration test
}
```

**Test File Organization**:

```
tests/
├── unit/                    (Unit tests - single functions/methods)
│   ├── network/
│   │   ├── test_client_creation.rs     (Tests NetworkClient::new)
│   │   ├── test_client_requests.rs     (Tests NetworkClient::request)
│   │   └── test_proxy_rotation.rs      (Tests ProxyRotator functions)
│   ├── crawler/
│   │   ├── test_engine_stats.rs        (Tests CrawlerEngine::statistics)
│   │   └── test_rate_limiting.rs       (Tests rate limiting functions)
├── integration/             (Integration tests - multiple components)
│   ├── test_crawler_engine.rs          (Full crawler workflow)
│   ├── test_storage_pipeline.rs        (Storage + processing)
│   └── test_network_stack.rs           (Network + proxy + client)
└── common/                  (Test utilities and fixtures)
    ├── fixtures.rs          (Test data and mock objects)
    ├── test_helpers.rs      (Helper functions for tests)
    └── mock_services.rs     (Mock implementations)
```

**Dependency Injection for Tests**:

```rust
// src/network/client.rs - Production code with injection points
pub struct NetworkClient {
    inner: Box<dyn HttpProvider>, // Trait for dependency injection
}

pub trait HttpProvider: Send + Sync {
    async fn get(&self, url: &str) -> Result<Response, NetworkError>;
}

// tests/common/mock_services.rs - Test implementations
pub struct MockHttpProvider {
    responses: HashMap<String, MockResponse>,
}

impl HttpProvider for MockHttpProvider {
    async fn get(&self, url: &str) -> Result<Response, NetworkError> {
        self.responses.get(url)
            .map(|mock| mock.to_response())
            .unwrap_or_else(|| Err(NetworkError::NotFound)) // OK in test helper
    }
}

// tests/unit/network/test_client_requests.rs
#[test]
async fn test_successful_request() {
    let mut mock_provider = MockHttpProvider::new();
    mock_provider.add_response("https://example.com", MockResponse::success("test"));

    let client = NetworkClient::with_provider(Box::new(mock_provider));
    let result = client.get("https://example.com").await.expect("Request should succeed"); // OK in tests

    assert_eq!(result.body, "test");
}
```

## 🚫 **FORBIDDEN ACTIONS**

- Using `unsafe` Rust without explicit approval and safety documentation
- Hardcoding configuration values anywhere
- Direct `crate::` or `std::` calls in function bodies
- Public exposure of sensitive fields
- Assembly of components outside `common/` folder
- Removing existing error handling
- Breaking backward compatibility
- **Using `unwrap()` or `expect()` in production code (enforced by clippy)**
- **Mixing tests with production code in the same file for complex modules**
- **Using dependency injection outside of test scenarios**
- Creating folder hierarchies deeper than 3 levels
- Using `for`/`while` loops when functional patterns are suitable
- Using `if-else` chains for 3+ conditions instead of `match`
- Holding locks during expensive operations
- Using blocking lock operations when non-blocking alternatives exist

## 📝 **Reporting Protocol**

When making changes, report:

- **What**: Files modified and nature of changes
- **Why**: Justification following these rules
- **Pattern**: Which architectural pattern was applied
- **Verification**: Test results and compilation status

---

_These rules ensure maintainable, secure, and well-architected Rust code_

### **7. 📂 Feature-Based Folder Organization**

**Rule**: Strict 3-level hierarchy with clear separation of concerns

```
Level 1: Root Features
├── network/          (Top-level feature)
├── crawler/          (Top-level feature)
├── storage/          (Top-level feature)
└── core/             (Top-level feature)

Level 2: Main Feature Modules (Assembly Only)
├── network/
│   ├── client/       (Main feature - assembles sub-features)
│   ├── proxy/        (Main feature - assembles sub-features)
│   └── mod.rs        (Exports assembled components)

Level 3: Sub-Features (Implementation)
├── network/
│   ├── client/
│   │   ├── builder.rs    (Self-contained: struct, methods, tests)
│   │   ├── config.rs     (Self-contained: struct, methods, tests)
│   │   └── mod.rs        (Assembles for Level 2)
│   └── proxy/
│       ├── provider.rs   (Self-contained: struct, methods, tests)
│       ├── rotation.rs   (Self-contained: struct, methods, tests)
│       └── mod.rs        (Assembles for Level 2)
```

**Level 2 Assembly Pattern**:

```rust
// network/client/mod.rs (Level 2 - Assembly Only)
mod builder;
mod config;

pub use builder::ClientBuilder;
pub use config::ClientConfig;

// Assemble components for external use
pub struct NetworkClient {
    config: ClientConfig,
    // Other assembled components
}

impl NetworkClient {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}
```

### **8. 🦀 Idiomatic Rust Enforcement**

**Rule**: Functional patterns over imperative, safe error handling

```rust
// ❌ BAD: Imperative loops and unsafe error handling
fn process_urls(urls: Vec<String>) -> Vec<String> {
    let mut results = Vec::new();
    for url in urls {
        if url.starts_with("https") {
            let processed = url.to_uppercase();
            results.push(processed);
        }
    }
    results
}

fn risky_operation() -> String {
    get_data().unwrap() // ❌ Can panic!
}

// ✅ GOOD: Functional patterns and safe error handling
fn process_urls(urls: &[String]) -> Vec<String> {
    urls.iter()
        .filter(|url| url.starts_with("https"))
        .map(|url| url.to_uppercase())
        .collect()
}

fn safe_operation() -> Result<String, ProcessingError> {
    get_data()
        .map_err(|e| ProcessingError::DataRetrievalFailed(e.to_string()))?
        .parse()
        .map_err(|e| ProcessingError::ParseFailed(e.to_string()))
}
```

**Custom Error Types Required**:

```rust
// core/errors.rs - Specific error handlers for different operations
use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Timeout after {duration}ms")]
    Timeout { duration: u64 },

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },
}

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("Failed to parse HTML: {reason}")]
    HtmlParsingFailed { reason: String },

    #[error("Rate limit exceeded for domain: {domain}")]
    RateLimitExceeded { domain: String },

    #[error("Crawl depth limit reached: {max_depth}")]
    DepthLimitReached { max_depth: usize },
}

// Usage with proper error propagation
impl NetworkClient {
    pub async fn fetch(&self, url: &str) -> Result<String, NetworkError> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| NetworkError::ConnectionFailed {
                message: e.to_string()
            })?;

        response
            .text()
            .await
            .map_err(|e| NetworkError::ConnectionFailed {
                message: format!("Failed to read response: {}", e)
            })
    }
}
```

### **9. 🔀 Match Over Complex Conditionals**

**Rule**: Use `match` instead of `if-else` chains for 3+ conditions

```rust
// ❌ BAD: Complex if-else chains
fn handle_status_code(code: u16) -> String {
    if code == 200 {
        "Success".to_string()
    } else if code >= 400 && code < 500 {
        "Client Error".to_string()
    } else if code >= 500 {
        "Server Error".to_string()
    } else {
        "Other".to_string()
    }
}

// ✅ GOOD: Pattern matching with exhaustive coverage
fn handle_status_code(code: u16) -> ResponseType {
    match code {
        200..=299 => ResponseType::Success,
        300..=399 => ResponseType::Redirect,
        400..=499 => ResponseType::ClientError,
        500..=599 => ResponseType::ServerError,
        _ => ResponseType::Unknown,
    }
}

// ✅ GOOD: Complex enum matching
fn process_crawler_result(result: CrawlResult) -> Action {
    match result {
        CrawlResult::Success { data, depth } if depth < MAX_DEPTH => {
            Action::Continue(data)
        },
        CrawlResult::Success { data, depth } => {
            Action::Complete(data)
        },
        CrawlResult::RateLimited { retry_after } => {
            Action::Delay(retry_after)
        },
        CrawlResult::Failed { error, retries } if retries < MAX_RETRIES => {
            Action::Retry(error)
        },
        CrawlResult::Failed { error, .. } => {
            Action::Abort(error)
        },
    }
}
```

### **10. 🔓 Fearless Concurrency**

**Rule**: Prefer non-blocking operations and minimal lock duration

```rust
use std::sync::{Arc, RwLock};
use tokio::sync::{Mutex, RwLock as TokioRwLock};

// ❌ BAD: Blocking lock operations
fn update_stats(stats: Arc<Mutex<Stats>>) {
    let mut stats = stats.lock().unwrap(); // Blocks!
    stats.increment();
    expensive_operation(); // Still holding lock!
}

// ✅ GOOD: Non-blocking with try_lock
fn update_stats_safe(stats: Arc<Mutex<Stats>>) -> Result<(), StatsError> {
    if let Ok(mut stats) = stats.try_lock() {
        stats.increment();
        // Lock released automatically here
        Ok(())
    } else {
        Err(StatsError::LockContention)
    }
}

// ✅ GOOD: Async-friendly concurrency
async fn update_stats_async(stats: Arc<TokioRwLock<Stats>>) -> Result<(), StatsError> {
    {
        let mut stats = stats.write().await;
        stats.increment();
        // Lock released when scope ends
    }

    // Do expensive work outside lock
    expensive_async_operation().await?;
    Ok(())
}

// ✅ GOOD: Read-heavy operations with RwLock
fn get_stats_snapshot(stats: Arc<RwLock<Stats>>) -> Option<StatsSnapshot> {
    stats.try_read()
        .ok()
        .map(|stats| stats.snapshot())
}
```

### **11. ⚖️ Performance-Aware Rule Enforcement**

**Rule**: Apply rules 7-10 unless performance, memory, or behavior issues arise

```rust
// Sometimes imperative is more efficient - ALLOWED
fn hot_path_processing(large_dataset: &[u64]) -> u64 {
    // Functional approach would create intermediate collections
    // let sum: u64 = large_dataset.iter().map(|x| x * 2).sum();

    // Imperative for performance-critical path - ACCEPTABLE
    let mut sum = 0;
    for &value in large_dataset {
        sum += value * 2;
    }
    sum
}

// Document performance exceptions
/// Performance-optimized loop - avoid iterator overhead for large datasets
/// Benchmark shows 30% improvement over functional approach
fn batch_process(items: &mut [Item]) {
    for item in items {
        item.process_in_place(); // Mutating in place for memory efficiency
    }
}

// Still prefer safety even in performance code
fn unsafe_but_documented(data: &[u8]) -> Result<ProcessedData, ProcessingError> {
    // Safety: We validate length before unsafe access
    if data.len() < 4 {
        return Err(ProcessingError::InsufficientData);
    }

    // SAFETY: Length checked above
    let header = unsafe {
        std::slice::from_raw_parts(data.as_ptr(), 4)
    };

    ProcessedData::from_header(header)
}
```

---

_These comprehensive rules ensure maintainable, performant, and idiomatic Rust code_
