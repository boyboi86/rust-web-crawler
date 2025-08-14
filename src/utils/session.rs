/// Session-related utility functions
use crate::session::manager::SessionResult;
use tracing::info;

/// Log a comprehensive session summary
pub fn log_session_summary(result: &SessionResult) {
    info!("=== Crawl Session Summary ===");
    info!("Session ID: {}", result.session_id);
    info!("Total URLs crawled: {}", result.total_urls_processed);

    info!("Successful crawls: {}", result.successful_crawls);
    info!("Failed crawls: {}", result.failed_crawls);

    if result.total_urls_processed > 0 {
        let success_rate =
            (result.successful_crawls as f64 / result.total_urls_processed as f64) * 100.0;
        info!("Success rate: {:.1}%", success_rate);
    }

    let total_words: usize = result
        .results
        .iter()
        .filter_map(|r| r.content.as_ref())
        .map(|c| c.word_count)
        .sum();
    info!("Total words crawled: {}", total_words);

    let avg_duration = if !result.results.is_empty() {
        let total_ms: u128 = result.results.iter().map(|r| r.duration.as_millis()).sum();
        total_ms / result.results.len() as u128
    } else {
        0
    };
    info!("Average response time: {}ms", avg_duration);

    // Language distribution
    let mut languages = std::collections::HashMap::new();
    for crawl_result in &result.results {
        if let Some(ref content) = crawl_result.content {
            if let Some(ref lang) = content.detected_language {
                *languages.entry(format!("{:?}", lang)).or_insert(0) += 1;
            }
        }
    }

    if !languages.is_empty() {
        info!("Language distribution:");
        for (lang, count) in languages {
            info!("  {}: {} pages", lang, count);
        }
    }

    info!(
        "Session duration: {:.2}s",
        result.total_duration.as_secs_f64()
    );
    info!("=== End Summary ===");
}
