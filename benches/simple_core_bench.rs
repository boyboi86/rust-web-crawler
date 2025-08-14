use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rust_web_crawler::core::{
    error::CrawlError,
    types::{CrawlTask, OptionInstant, TaskPriority, TaskTiming},
    utils::ErrorUtils,
};
use std::time::Duration;
use url::Url;

/// Benchmark our refactored building block approach vs original approach
fn benchmark_task_creation(c: &mut Criterion) {
    let url = Url::parse("https://example.com").unwrap();

    c.bench_function("task_creation_with_building_blocks", |b| {
        b.iter(|| {
            black_box(CrawlTask::new(
                black_box(url.clone()),
                black_box(TaskPriority::Normal),
                black_box(3),
            ))
        })
    });
}

/// Benchmark scoped variable approach in error categorization
fn benchmark_error_categorization(c: &mut Criterion) {
    let test_error = anyhow::anyhow!("Connection timeout after 30 seconds");

    c.bench_function("scoped_variable_error_categorization", |b| {
        b.iter(|| black_box(CrawlError::from_anyhow_error(black_box(&test_error))))
    });
}

/// Benchmark functional URL validation approach
fn benchmark_functional_url_validation(c: &mut Criterion) {
    let test_urls = vec![
        "https://example.com/page",
        "http://test.org/path/to/resource",
        "https://invalid.com/file.pdf",
        "ftp://invalid.scheme.com",
    ];

    c.bench_function("functional_url_validation", |b| {
        b.iter(|| {
            for url in &test_urls {
                black_box(ErrorUtils::is_valid_crawl_url(black_box(url)));
            }
        })
    });
}

/// Benchmark TaskTiming building block operations
fn benchmark_task_timing_operations(c: &mut Criterion) {
    c.bench_function("task_timing_building_block", |b| {
        b.iter(|| {
            let mut timing = black_box(TaskTiming::new());
            timing.mark_started();
            timing.mark_attempt();
            timing.set_retry_delay(Duration::from_millis(1000));
            black_box(timing.is_ready_for_retry());
        })
    });
}

/// Benchmark type alias usage
fn benchmark_option_instant_usage(c: &mut Criterion) {
    c.bench_function("option_instant_type_alias", |b| {
        b.iter(|| {
            let _instant: OptionInstant = Some(std::time::Instant::now());
            black_box(_instant)
        })
    });
}

criterion_group!(
    benches,
    benchmark_task_creation,
    benchmark_error_categorization,
    benchmark_functional_url_validation,
    benchmark_task_timing_operations,
    benchmark_option_instant_usage
);
criterion_main!(benches);
