# Additional Coding Standards 7-11 - Explanation & Implementation

## Overview

Added 5 comprehensive coding standards (Rules 7-11) to the existing AI Agent Rules, focusing on project organization, idiomatic Rust practices, and performance-aware development.

## 📋 **Detailed Rule Explanations**

### **Rule 7: 📂 Feature-Based Folder Organization**

**Purpose**: Enforce clear separation of concerns with strict hierarchy limits

**Key Principles**:

- **Level 1**: Root features (network/, crawler/, storage/, core/)
- **Level 2**: Main feature modules (network/client/, network/proxy/) - **Assembly Only**
- **Level 3**: Sub-features (client/builder.rs, client/config.rs) - **Implementation**

**Critical Rule**: Level 2 modules should ONLY assemble Level 3 components, never implement business logic directly.

**Example Structure**:

```
src/
├── network/                    (Level 1)
│   ├── client/                 (Level 2 - Assembly)
│   │   ├── builder.rs          (Level 3 - Implementation)
│   │   ├── config.rs           (Level 3 - Implementation)
│   │   └── mod.rs              (Assembles components)
│   ├── proxy/                  (Level 2 - Assembly)
│   │   ├── provider.rs         (Level 3 - Implementation)
│   │   ├── rotation.rs         (Level 3 - Implementation)
│   │   └── mod.rs              (Assembles components)
│   └── mod.rs                  (Exports to main)
```

### **Rule 8: 🦀 Idiomatic Rust Enforcement**

**Purpose**: Ensure functional programming patterns and safe error handling

**Mandated Practices**:

- **Use**: `.map()`, `.filter()`, `.collect()`, method chaining
- **Avoid**: `for` loops, `while` loops when functional alternatives exist
- **Forbid**: `unwrap()`, `expect()` in production code
- **Require**: `Result<T, E>`, `Option<T>`, `?` operator for error propagation
- **Custom Errors**: Use `anyhow` + `thiserror` for specific error types

**Error Handler Requirements**:

```rust
// Required: Specific error types for different operations
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Timeout after {duration}ms")]
    Timeout { duration: u64 },
}

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("Failed to parse HTML: {reason}")]
    HtmlParsingFailed { reason: String },

    #[error("Rate limit exceeded for domain: {domain}")]
    RateLimitExceeded { domain: String },
}
```

### **Rule 9: 🔀 Match Over Complex Conditionals**

**Purpose**: Improve readability and leverage Rust's pattern matching

**Rule**: Use `match` statements instead of `if-else` chains when handling 3+ conditions

**Benefits**:

- Exhaustiveness checking by compiler
- Better readability
- More maintainable code
- Leverages Rust's powerful pattern matching

**Example**:

```rust
// ❌ BAD: Complex if-else chain
if status == 200 {
    "Success"
} else if status >= 400 && status < 500 {
    "Client Error"
} else if status >= 500 {
    "Server Error"
} else {
    "Other"
}

// ✅ GOOD: Pattern matching
match status {
    200..=299 => ResponseType::Success,
    300..=399 => ResponseType::Redirect,
    400..=499 => ResponseType::ClientError,
    500..=599 => ResponseType::ServerError,
    _ => ResponseType::Unknown,
}
```

### **Rule 10: 🔓 Fearless Concurrency**

**Purpose**: Minimize lock contention and promote non-blocking operations

**Key Principles**:

- **Prefer**: `try_lock()` over `lock()`
- **Minimize**: Lock duration (scope locks tightly)
- **Use**: Async-friendly concurrency primitives (`tokio::sync`)
- **Avoid**: Holding locks during expensive operations

**Pattern Example**:

```rust
// ✅ GOOD: Minimal lock duration
async fn update_stats(stats: Arc<TokioRwLock<Stats>>) -> Result<(), StatsError> {
    {
        let mut stats = stats.write().await;
        stats.increment();
        // Lock automatically released here
    }

    // Expensive work done outside lock
    expensive_async_operation().await?;
    Ok(())
}
```

### **Rule 11: ⚖️ Performance-Aware Rule Enforcement**

**Purpose**: Allow rule flexibility when performance demands it

**Conditions for Rule Relaxation**:

- **Performance**: Measurable degradation (>10% slower)
- **Memory**: Significant increase in memory usage
- **Behavior**: Unexpected or incorrect program behavior

**Requirements When Relaxing Rules**:

- **Document**: Clear comments explaining why
- **Benchmark**: Provide performance justification
- **Safety**: Maintain memory safety even in optimized code

**Example**:

```rust
/// Performance-optimized loop - avoid iterator overhead for large datasets
/// Benchmark shows 30% improvement over functional approach
fn hot_path_processing(large_dataset: &[u64]) -> u64 {
    // Imperative for performance-critical path - ACCEPTABLE
    let mut sum = 0;
    for &value in large_dataset {
        sum += value * 2;
    }
    sum
}
```

## 🔄 **Updated AI Agent Rules Document**

### **Comprehensive Changes Made**:

1. **Updated Title**: "MANDATORY CODING STANDARDS (11 Rules)"
2. **Added 5 New Sections**: Rules 7-11 with detailed examples
3. **Enhanced Forbidden Actions**: Added new restrictions aligned with the rules
4. **Error Handling**: Comprehensive error type requirements
5. **Concurrency Patterns**: Detailed async/sync best practices
6. **Performance Guidelines**: When and how to optimize responsibly

### **Documentation Structure**:

```
📋 MANDATORY CODING STANDARDS (11 Rules)
├── Rules 1-6: (Existing - Architecture & Design)
│   ├── No Hardcoding
│   ├── Explicit Imports
│   ├── Builder Patterns
│   ├── Privacy First
│   ├── Composition
│   └── External Usage
├── Rules 7-11: (New - Organization & Implementation)
│   ├── Feature-Based Organization
│   ├── Idiomatic Rust
│   ├── Match Over If-Else
│   ├── Fearless Concurrency
│   └── Performance-Aware Enforcement
└── Enhanced Forbidden Actions List
```

## 🎯 **Implementation Guidelines**

### **For AI Agents**:

1. **Always Review**: Check existing code structure before implementing new features
2. **Follow Hierarchy**: Respect the 3-level folder organization
3. **Error First**: Define specific error types before implementing functionality
4. **Test Patterns**: Use the established patterns from `common/building_blocks.rs`
5. **Performance Conscious**: Measure before optimizing, document exceptions

### **For Developers**:

1. **Migration Path**: Existing code should gradually adopt these patterns
2. **Consistency**: New code must follow all 11 rules from day one
3. **Code Reviews**: Use rules as checklist items
4. **Documentation**: Update module docs to reflect architectural decisions

## 📊 **Verification Checklist**

Before code submission, verify:

- [ ] ✅ No hardcoded values (Rule 1)
- [ ] ✅ Explicit imports (Rule 2)
- [ ] ✅ Builder patterns for complex structs (Rule 3)
- [ ] ✅ Private fields with controlled access (Rule 4)
- [ ] ✅ Assembly only in `common/` (Rule 5)
- [ ] ✅ External usage through builders (Rule 6)
- [ ] ✅ Max 3-level folder depth (Rule 7)
- [ ] ✅ Functional patterns over loops (Rule 8)
- [ ] ✅ Match for 3+ conditions (Rule 9)
- [ ] ✅ Non-blocking concurrency (Rule 10)
- [ ] ✅ Performance-justified exceptions documented (Rule 11)

---

The comprehensive 11-rule system now provides complete guidance for building maintainable, performant, and idiomatic Rust applications while maintaining architectural consistency and safety.
