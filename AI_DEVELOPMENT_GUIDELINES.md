# AI Agent Development Guidelines

## 🤖 Rules for AI Agents Working on This Rust Project

### ✅ **ALLOWED ACTIONS** (No Permission Required)

#### Code Development

- Read any file in the project for context
- Create new `.rs` files in the `src/` directory following existing patterns
- Modify existing `.rs` files for bug fixes and improvements
- Add comprehensive error handling and documentation
- Create unit tests in appropriate locations
- Update inline documentation and comments

#### Dependencies & Building

- Add new dependencies to `Cargo.toml` (with clear justification)
- Run development commands: `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`
- Create examples in the `examples/` directory
- Update version numbers for patch/minor releases

#### Configuration

- Use existing configuration system in `config/` module
- Add new constants to `config/constants.rs` (avoid hardcoding)
- Modify default configurations for new features

### 🔒 **REQUIRES PERMISSION** (Ask Before Proceeding)

#### Structural Changes

- Deleting any existing files or modules
- Moving modules or changing project structure
- Modifying the 4-layer architecture (api, actors, core, utils)
- Breaking changes to public APIs

#### Dependencies & Security

- Major version updates of dependencies
- Adding dependencies with potential security implications
- Removing existing dependencies

#### Configuration & Deployment

- Modifying `.gitignore`, CI/CD workflows, or Docker files
- Changing security-related configurations
- Running `cargo publish` or deployment commands
- Modifying Tauri configuration files

### 📋 **MUST FOLLOW** (Non-Negotiable Standards)

#### Rust Conventions

- Use snake_case for functions, variables, and modules
- Use PascalCase for types, structs, and enums
- Follow Rust 2021 edition guidelines
- Maximum line length: 100 characters

#### Code Quality

- Add comprehensive error handling with `Result<T, E>` types
- Include documentation comments (`///`) for all public APIs
- Write unit tests for new functionality
- Handle all compiler warnings (no ignored warnings)
- Use structured error types, not string errors

#### Project Architecture

- Follow the existing 4-layer pattern:
  - `api/` - Tauri command interfaces
  - `actors/` - Async actor patterns for concurrency
  - `core/` - Core types and business logic
  - `utils/` - Shared utilities and validation
- Use the centralized configuration system
- Maintain separation of concerns

#### Configuration Management

- Never hardcode configuration values
- Add new constants to `config/constants.rs`
- Use the existing environment variable system
- Respect the preset configurations (dev/prod/test)

### 🚫 **FORBIDDEN** (Never Do These)

#### Unsafe Practices

- Using `unsafe` Rust without explicit permission and documentation
- Ignoring or suppressing compiler errors
- Using `unwrap()` or `expect()` in production code paths
- Adding dependencies with known security vulnerabilities

#### Breaking Changes

- Removing existing public APIs
- Changing function signatures of public methods
- Removing existing error handling
- Breaking backward compatibility

#### Code Quality Issues

- Hardcoding values that should be configurable
- Removing existing unit tests
- Adding TODO comments without creating issues
- Using deprecated Rust features

### 🏗️ **Development Workflow**

1. **Before Making Changes:**

   - Read relevant files to understand context
   - Check existing tests and documentation
   - Verify the change aligns with project architecture

2. **While Developing:**

   - Run `cargo check` frequently during development
   - Add appropriate error handling
   - Include documentation for new public APIs
   - Follow existing naming and structure patterns

3. **Before Completion:**
   - Run `cargo test` to ensure no regressions
   - Run `cargo clippy` to catch common issues
   - Run `cargo fmt` to format code consistently
   - Verify all compiler warnings are addressed

### 🎯 **Specific Project Patterns**

#### Error Handling

```rust
// ✅ Preferred: Structured errors
pub enum CrawlerError {
    NetworkError(String),
    ConfigurationError(String),
    ValidationError(String),
}

// ❌ Avoid: String errors
fn bad_function() -> Result<(), &'static str>
```

#### Configuration Usage

```rust
// ✅ Use constants
use crate::config::constants::DEFAULT_MAX_RETRIES;

// ❌ Don't hardcode
let max_retries = 3; // Bad!
```

#### Async Patterns

```rust
// ✅ Follow existing actor patterns
// Use CrawlerActor for WebCrawler integration
// Maintain Send/Sync boundaries properly
```

### 📝 **Communication Guidelines**

#### When Asking for Permission:

- Clearly explain what you want to do and why
- Describe potential risks or impacts
- Suggest alternatives if the action is denied
- Provide context about the benefit

#### When Reporting Completion:

- Summarize what was changed
- Note any new dependencies or requirements
- Highlight any potential impacts
- Mention test results and verification steps

---

_This document serves as a guide for AI agents working on this project. Following these guidelines ensures code quality, security, and maintainability._
