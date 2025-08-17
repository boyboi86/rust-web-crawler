# AI Agent Configuration Schema

This file defines the configuration schema that AI agents should understand when working on this project.

## Project Metadata

```json
{
  "project": {
    "name": "rust_web_crawler",
    "type": "rust_tauri_application",
    "architecture": "4_layer_pattern",
    "primary_language": "rust",
    "frontend": "typescript_react"
  }
}
```

## AI Agent Permissions Matrix

| Action              | Permission Level         | Notes                        |
| ------------------- | ------------------------ | ---------------------------- |
| Read files          | ✅ Always Allowed        | Full read access for context |
| Create .rs files    | ✅ Always Allowed        | Following existing patterns  |
| Modify .rs files    | ✅ Always Allowed        | For improvements and fixes   |
| Add dependencies    | ⚠️ Require Justification | Explain why needed           |
| Delete files        | 🔒 Require Permission    | Must ask first               |
| Modify Cargo.toml   | ⚠️ Require Justification | Document changes             |
| Change architecture | 🔒 Require Permission    | Major structural changes     |
| Deploy/Publish      | 🚫 Forbidden             | Human-only action            |
| Security configs    | 🔒 Require Permission    | Sensitive operations         |

## Code Quality Standards

### Required Patterns

- **Error Handling**: All functions return `Result<T, E>`
- **Documentation**: Public APIs must have `///` comments
- **Testing**: New features require unit tests
- **Configuration**: Use `config/constants.rs`, no hardcoding
- **Naming**: Follow Rust conventions (snake_case, PascalCase)

### Architecture Constraints

- **Layer Separation**: Respect api/actors/core/utils boundaries
- **Async Patterns**: Use existing actor patterns for concurrency
- **Type Safety**: Prefer compile-time checks over runtime
- **Send/Sync**: Maintain thread safety boundaries

### Development Workflow

1. **Pre-change**: Read context, understand architecture
2. **During**: Run `cargo check` frequently, add tests
3. **Post-change**: Run full validation workflow
4. **Verification**: Ensure no breaking changes

## Validation Commands

AI agents should run these before considering work complete:

```bash
# Development checks
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo fmt --check

# Security and quality
cargo audit  # If cargo-audit is installed
```

## Emergency Stops

If any of these conditions are met, STOP and ask for permission:

- Compilation errors after changes
- Test failures in existing tests
- Clippy errors with deny-level warnings
- Breaking changes to public APIs
- Security audit failures

## Communication Protocol

### When Requesting Permission:

```
🔒 Permission Request: [Brief Action Description]

**What**: [Detailed explanation]
**Why**: [Justification]
**Risk**: [Potential impacts]
**Alternative**: [Other approaches considered]
```

### When Reporting Completion:

```
✅ Task Complete: [Brief Description]

**Changes**: [What was modified]
**Tests**: [Test results]
**Verification**: [cargo check/clippy/test status]
**Impact**: [Any breaking changes or new requirements]
```
