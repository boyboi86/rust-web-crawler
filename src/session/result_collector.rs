use std::collections::HashMap;

use super::manager::CrawlResultData;

/// Collects and manages crawl results during a session
pub struct ResultCollector {
    results: Vec<CrawlResultData>,
    url_status: HashMap<String, ResultStatus>,
}

#[derive(Debug, Clone)]
pub enum ResultStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

impl ResultCollector {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            url_status: HashMap::new(),
        }
    }

    /// Mark a URL as being processed
    pub fn mark_in_progress(&mut self, url: &str) {
        self.url_status
            .insert(url.to_string(), ResultStatus::InProgress);
    }

    /// Add a completed result
    pub fn add_result(&mut self, result: CrawlResultData) {
        let status = if result.content.is_some() {
            ResultStatus::Completed
        } else {
            ResultStatus::Failed(result.error.clone().unwrap_or_default())
        };

        self.url_status.insert(result.url.to_string(), status);
        self.results.push(result);
    }

    /// Get all collected results
    pub fn get_results(&self) -> &[CrawlResultData] {
        &self.results
    }

    /// Get status of a specific URL
    pub fn get_url_status(&self, url: &str) -> Option<&ResultStatus> {
        self.url_status.get(url)
    }

    /// Get count of results by status
    pub fn get_status_counts(&self) -> StatusCounts {
        let mut counts = StatusCounts::default();

        for status in self.url_status.values() {
            match status {
                ResultStatus::Pending => counts.pending += 1,
                ResultStatus::InProgress => counts.in_progress += 1,
                ResultStatus::Completed => counts.completed += 1,
                ResultStatus::Failed(_) => counts.failed += 1,
            }
        }

        counts
    }
}

#[derive(Debug, Default, Clone)]
pub struct StatusCounts {
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
}

impl StatusCounts {
    pub fn total(&self) -> usize {
        self.pending + self.in_progress + self.completed + self.failed
    }

    pub fn success_rate(&self) -> f64 {
        let total_finished = self.completed + self.failed;
        if total_finished == 0 {
            0.0
        } else {
            (self.completed as f64 / total_finished as f64) * 100.0
        }
    }
}
