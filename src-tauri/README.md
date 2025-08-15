# Tauri Desktop Application - Rust Backend

A desktop application built with **Tauri 2.7.0** that provides a TypeScript frontend interface to the Rust Web Crawler. This application implements a robust **Actor Pattern** architecture to bridge the gap between Tauri's thread-safe requirements and the web crawler's non-Send types.

## 🏗️ Architecture Overview

This Tauri application follows a **4-layer architecture pattern** for maintainability, scalability, and clear separation of concerns:

```
src-tauri/src/
├── 📡 api/              # API Layer - Tauri Command Handlers
├── 🎭 actors/           # Actor Pattern - Concurrency & Send/non-Send Bridging
├── 🎯 core/             # Core Data Layer - Types & Models
├── 🔧 utils/            # Utilities Layer - Validation & Helpers
├── 📚 lib.rs            # Main Application Orchestration
└── 🚀 main.rs           # Application Entry Point
```

## 📁 Detailed Folder Structure

### 📡 **API Layer** (`api/`)

**Purpose**: Tauri command handlers that provide the public API interface between TypeScript frontend and Rust backend.

```
api/
├── mod.rs          # API module orchestration and re-exports
└── commands.rs     # Tauri command implementations
```

**Key Commands**:

- `get_default_config()` - Returns default crawler configuration
- `validate_config(request)` - Validates crawl request parameters
- `start_crawl(request)` - Initiates a new crawl session
- `get_crawl_status(session_id)` - Retrieves current crawl status
- `stop_crawl(session_id)` - Stops an active crawl session

### 🎭 **Actors Layer** (`actors/`)

**Purpose**: Implements the Actor Pattern to manage concurrency and bridge between Tauri's Send-safe environment and the web crawler's non-Send types.

```
actors/
├── mod.rs              # Actor module orchestration
└── crawler_actor.rs    # Core actor implementation
```

**Key Components**:

- `CrawlerBridge` - Send-safe interface for Tauri commands
- `CrawlerActor` - Dedicated thread actor that owns the WebCrawler
- `ActorMessage` - Message types for actor communication
- Message queue system using `tokio::sync::mpsc` channels

**Architecture Solution**:

```
┌─────────────────┐    Messages    ┌──────────────────┐    WebCrawler    ┌─────────────────┐
│ Tauri Commands  │ ──────────────> │ CrawlerBridge    │ ─────────────────> │ CrawlerActor    │
│ (Send + Sync)   │                 │ (Send-safe)      │                   │ (non-Send safe) │
└─────────────────┘                 └──────────────────┘                   └─────────────────┘
```

### 🎯 **Core Layer** (`core/`)

**Purpose**: Contains all core data structures, types, and models used throughout the application.

```
core/
├── mod.rs      # Core module orchestration
└── types.rs    # Data structures and models
```

**Key Types**:

- `CrawlRequest` - Frontend crawl configuration request
- `CrawlStatus` - Real-time crawl session status
- `CrawlResultSummary` - Summary of crawled page results

### 🔧 **Utils Layer** (`utils/`)

**Purpose**: Utility functions, validation logic, and helper functions.

```
utils/
├── mod.rs          # Utils module orchestration
├── validation.rs   # Input validation and error handling
└── helpers.rs      # Helper functions (minimal)
```

**Key Functions**:

- `validate_crawl_request()` - Comprehensive request validation
- `ValidationError` - Custom error type for validation failures

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.77.2 or higher
- **Node.js**: 18+ (for frontend)
- **Tauri CLI**: Latest version

### Installation

```bash
# Install Tauri CLI if not already installed
cargo install tauri-cli

# Navigate to the Tauri directory
cd src-tauri

# Install dependencies
cargo build
```

### Development

```bash
# Run in development mode
cargo tauri dev

# Build for production
cargo tauri build

# Run tests
cargo test

# Check code without building
cargo check
```

## 📦 Dependencies

### Core Dependencies

- **Tauri 2.7.0** - Desktop application framework
- **Tokio 1.0** - Async runtime with full features
- **Serde 1.0** - Serialization framework
- **URL 2.4** - URL parsing and validation

### Logging & Utilities

- **log 0.4** - Logging facade
- **env_logger 0.10** - Environment-based logging
- **fastrand 2.0** - Fast random number generation

### Integration

- **rust_web_crawler** - Local path dependency to main crawler library

## 🎯 Key Features

### ✅ **Thread-Safe Actor Pattern**

- Solves Send trait incompatibility between Tauri and WebCrawler
- Isolates non-Send types in dedicated actor thread
- Provides async, Send-safe interface for Tauri commands

### ✅ **Robust Session Management**

- Unique session ID generation and tracking
- Real-time status updates and progress monitoring
- Error handling and session cleanup

### ✅ **Comprehensive Validation**

- URL format validation
- Parameter range checking
- Custom error types with field-specific messages

### ✅ **Clean Architecture**

- 4-layer separation of concerns
- Modular design for easy maintenance
- Clear public API surface

## 🔧 Configuration

### Tauri Configuration

Located in `tauri.conf.json` - handles window settings, security, and build configuration.

### Crawler Configuration

Default configuration provided through `get_default_config()` command, customizable via frontend interface.

## 🐛 Debugging

### Development Mode

```bash
cargo tauri dev
```

Enables:

- Hot reload for Rust code changes
- Debug logging output
- Developer tools in the webview

### Logging

The application uses structured logging:

- **Info level**: General operation flow
- **Error level**: Critical failures and validation errors
- **Debug level**: Detailed actor message flow (dev mode only)

### Common Issues

1. **Send Trait Violations**: The actor pattern specifically handles this - ensure all WebCrawler interactions go through the actor.

2. **Session Not Found**: Verify session IDs are properly generated and tracked.

3. **Validation Failures**: Check the validation module for specific field requirements.

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific module
cargo test actors::

# Check for compilation issues
cargo check
```

## 📈 Performance Considerations

- **Actor Isolation**: Non-Send WebCrawler runs in dedicated thread
- **Async Commands**: All Tauri commands are fully async
- **Message Queue**: Efficient communication via unbounded channels
- **Memory Management**: Automatic session cleanup on completion

## 🔒 Security

- **Input Validation**: All requests validated before processing
- **URL Validation**: Proper URL parsing and validation
- **Error Handling**: Structured error responses without sensitive data exposure

## 🛠️ Contributing

1. Follow the 4-layer architecture pattern
2. Ensure all new code compiles without warnings
3. Add appropriate validation for new request fields
4. Maintain the actor pattern for WebCrawler interactions
5. Update this README for new features

## 📝 License

This project is part of the Rust Web Crawler suite. See the main project license for details.

---

**Built with ❤️ using Tauri, Rust, and the Actor Pattern for seamless desktop web crawling.**
