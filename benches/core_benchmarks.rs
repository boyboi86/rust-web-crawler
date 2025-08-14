use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rust_web_crawler::core::{error::CrawlError, types::*, utils::ErrorUtils};
use std::time::Duration;
use url::Url;

/// Benchmark current vs refactored error handling patterns
fn benchmark_error_categorization(c: &mut Criterion) {
    let test_error = anyhow::anyhow!("Connection timeout after 30 seconds");

    c.bench_function("current_error_categorization", |b| {
        b.iter(|| black_box(CrawlError::from_anyhow_error(black_box(&test_error))))
    });

    c.bench_function("error_utils_categorization", |b| {
        b.iter(|| {
            black_box(ErrorUtils::categorize_and_check_retry(black_box(
                &test_error,
            )))
        })
    });
}

/// Benchmark functional vs imperative URL validation
fn benchmark_url_validation(c: &mut Criterion) {
    let test_urls = vec![
        "https://example.com/page",
        "http://test.org/path/to/resource",
        "https://invalid.com/file.pdf",
        "ftp://invalid.scheme.com",
        "https://example.com/redirect/redirect/redirect/very/long/path/that/might/be/suspicious",
    ];

    c.bench_function("current_url_validation", |b| {
        b.iter(|| {
            for url in &test_urls {
                black_box(ErrorUtils::is_valid_crawl_url(black_box(url)));
            }
        })
    });

    // We'll add the refactored version comparison later
}

/// Benchmark CrawlTask creation patterns
fn benchmark_task_creation(c: &mut Criterion) {
    let url = Url::parse("https://example.com").unwrap();

    c.bench_function("current_task_creation", |b| {
        b.iter(|| {
            black_box(CrawlTask::new(
                black_box(url.clone()),
                black_box(TaskPriority::Normal),
                black_box(3),
            ))
        })
    });

    // We'll add building block version comparison later
}

/// Benchmark language detection patterns
fn benchmark_language_detection(c: &mut Criterion) {
    let content = "This is a sample English text for language detection testing purposes.";

    c.bench_function("simple_language_detection", |b| {
        b.iter(|| black_box(ErrorUtils::detect_language_simple(black_box(content))))
    });

    c.bench_function("typed_language_detection", |b| {
        b.iter(|| black_box(ErrorUtils::detect_language_typed(black_box(content))))
    });
}

/// Benchmark retry delay calculations
fn benchmark_retry_calculations(c: &mut Criterion) {
    let base_delay = Duration::from_millis(1000);
    let max_delay = Duration::from_millis(30000);

    c.bench_function("retry_delay_calculation", |b| {
        b.iter(|| {
            for attempt in 0..5 {
                black_box(ErrorUtils::calculate_retry_delay(
                    black_box(attempt),
                    black_box(base_delay),
                    black_box(max_delay),
                    black_box(2.0),
                ));
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_error_categorization,
    benchmark_url_validation,
    benchmark_task_creation,
    benchmark_language_detection,
    benchmark_retry_calculations
);
criterion_main!(benches);
