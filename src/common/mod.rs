//! Common building blocks and utilities
//!
//! This module contains reusable components that follow DRY principles
//! and provide single source of truth for common patterns.
//! Following Rule 5: Composition - assembling smaller building blocks

// Core building blocks (decomposed into http_client)
mod configuration;
mod primitives;
mod results;
mod statistics;
mod tasks;
mod timing;
pub use configuration::*;
pub use primitives::*;
pub use results::*;
pub use statistics::*;
pub use tasks::*;
pub use timing::*;

// Re-export http_client building blocks for convenience
pub mod http_client;
pub use http_client::api_params::{ApiParameterSet, BasicApiParam};
pub use http_client::reqwest_client::{ReqwestClient, ReqwestClientBuilder};
pub use http_client::response_validator::ResponseValidator;
pub use http_client::retry_policy::RetryPolicy;

// Re-export primitives
pub use primitives::{
    AttemptCount, BooleanFlag, CountValue, DelayDuration, DomainString, LimitValue, OptionalVec,
    PercentageValue, PriorityScore, SessionId, TaskId, TimeoutDuration, UrlString,
};

// Re-export timing patterns
pub use timing::{ExecutionTiming, PerformanceTiming, RateLimitTiming, TaskTiming, TimingConfig};

// Re-export statistics patterns
pub use statistics::{
    DomainStats, LimitTracker, PerformanceMetrics, TaskCounts, WindowEventType, WindowedStats,
};

// Re-export configuration patterns from Level 3 submodules
pub use configuration::config_builder::ConfigBuilder;
pub use configuration::content_filter_config::ContentFilterConfig;
pub use configuration::crawler_config::CrawlerConfig;
pub use configuration::http_client_config::HttpClientConfig;
pub use configuration::proxy_auth::ProxyAuth;
pub use configuration::proxy_config::ProxyConfig;
pub use configuration::proxy_type::ProxyType;
pub use configuration::rate_limit_config::RateLimitConfig;
pub use configuration::retry_config::RetryConfig;
pub use configuration::single_proxy_config::{SingleProxyConfig, SingleProxyConfigBuilder};

// Re-export result patterns
pub use results::{
    ConfigResult, CrawlResult, ErrorSeverity, NetworkResult, ProcessingResult, SessionResult,
    TaskContent, TaskError, TaskResult,
};

// Re-export task patterns
pub use tasks::{CrawlTask, CrawlTaskBuilder, TaskContext, TaskIdentity, TaskPriority, TaskStatus};
