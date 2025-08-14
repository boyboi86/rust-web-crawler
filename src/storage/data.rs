use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::hash::{DefaultHasher, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;

/// Data storage and output management
pub struct DataStorage {
    output_dir: PathBuf,
    format: OutputFormat,
    compression: bool,
}

#[derive(Debug)]
pub enum OutputFormat {
    Json,
    Jsonl, // JSON Lines
    Csv,
    Parquet,
}

/// Crawl result for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCrawlResult {
    pub url: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub word_count: usize,
    pub language: Option<String>,
    pub links_found: Vec<String>,
    pub metadata: CrawlMetadata,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlMetadata {
    pub status_code: Option<u16>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub response_time_ms: u64,
    pub depth: usize,
    pub parent_url: Option<String>,
    pub crawl_session_id: String,
}

/// Custom formatter trait for extensible output formats
pub trait CustomFormatter {
    fn format_result(&self, result: &StoredCrawlResult) -> Result<String>;
    fn get_file_extension(&self) -> &str;
    fn supports_streaming(&self) -> bool;
}

impl DataStorage {
    pub fn new<P: AsRef<Path>>(output_dir: P, format: OutputFormat) -> Result<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&output_dir)?;

        Ok(Self {
            output_dir,
            format,
            compression: false,
        })
    }

    /// Enable compression for output files
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }

    /// Store a single crawl result
    pub async fn store_result(&self, result: &StoredCrawlResult) -> Result<()> {
        let filename = self.generate_filename(&result.url, &result.timestamp);
        let filepath = self.output_dir.join(filename);

        match &self.format {
            OutputFormat::Json => {
                let content = serde_json::to_string_pretty(result)?;
                self.write_to_file(&filepath, content).await?;
            }
            OutputFormat::Jsonl => {
                let content = serde_json::to_string(result)?;
                self.append_to_file(&filepath, format!("{}\n", content))
                    .await?;
            }
            OutputFormat::Csv => {
                self.store_as_csv(result, &filepath).await?;
            }
            OutputFormat::Parquet => {
                return Err(anyhow::anyhow!("Parquet format not yet implemented"));
            }
        }

        Ok(())
    }

    /// Store multiple results in batch
    pub async fn store_batch(&self, results: &[StoredCrawlResult]) -> Result<()> {
        match &self.format {
            OutputFormat::Jsonl => {
                let filename = format!(
                    "batch_{}.jsonl",
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)?
                        .as_secs()
                );
                let filepath = self.output_dir.join(filename);

                let mut content = String::new();
                for result in results {
                    content.push_str(&serde_json::to_string(result)?);
                    content.push('\n');
                }

                self.write_to_file(&filepath, content).await?;
            }
            OutputFormat::Json => {
                let filename = format!(
                    "batch_{}.json",
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)?
                        .as_secs()
                );
                let filepath = self.output_dir.join(filename);

                let content = serde_json::to_string_pretty(results)?;
                self.write_to_file(&filepath, content).await?;
            }
            _ => {
                // For other formats, store individually
                for result in results {
                    self.store_result(result).await?;
                }
            }
        }

        Ok(())
    }

    /// Store crawl session summary
    pub async fn store_session_summary(
        &self,
        session_id: &str,
        summary: &CrawlSessionSummary,
    ) -> Result<()> {
        let filename = format!("session_summary_{}.json", session_id);
        let filepath = self.output_dir.join(filename);

        let content = serde_json::to_string_pretty(summary)?;
        self.write_to_file(&filepath, content).await?;

        Ok(())
    }

    /// Load stored results for analysis
    pub async fn load_results(&self, pattern: Option<&str>) -> Result<Vec<StoredCrawlResult>> {
        let mut results = Vec::new();
        let mut entries = fs::read_dir(&self.output_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if let Some(pattern) = pattern
                && !path.to_string_lossy().contains(pattern)
            {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).await?;

                // Try to parse as single result or array
                if let Ok(result) = serde_json::from_str::<StoredCrawlResult>(&content) {
                    results.push(result);
                } else if let Ok(batch) = serde_json::from_str::<Vec<StoredCrawlResult>>(&content) {
                    results.extend(batch);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                let content = fs::read_to_string(&path).await?;
                for line in content.lines() {
                    if let Ok(result) = serde_json::from_str::<StoredCrawlResult>(line) {
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Generate analytics from stored results
    pub async fn generate_analytics(&self) -> Result<CrawlAnalytics> {
        let results = self.load_results(None).await?;

        let mut analytics = CrawlAnalytics {
            total_pages: results.len(),
            ..Default::default()
        };

        let mut domain_counts = HashMap::new();
        let mut language_counts = HashMap::new();
        let mut total_words = 0;
        let mut total_response_time = 0u64;

        for result in &results {
            // Extract domain
            if let Ok(url) = url::Url::parse(&result.url)
                && let Some(host) = url.host_str()
            {
                *domain_counts.entry(host.to_string()).or_insert(0) += 1;
            }

            // Language statistics
            if let Some(lang) = &result.language {
                *language_counts.entry(lang.clone()).or_insert(0) += 1;
            }

            total_words += result.word_count;
            total_response_time += result.metadata.response_time_ms;

            // Status code statistics
            if let Some(status) = result.metadata.status_code {
                if (200..300).contains(&status) {
                    analytics.successful_crawls += 1;
                } else {
                    analytics.failed_crawls += 1;
                }
            }
        }

        analytics.domains_crawled = domain_counts.len();
        analytics.avg_words_per_page = if !results.is_empty() {
            total_words / results.len()
        } else {
            0
        };
        analytics.avg_response_time_ms = if !results.is_empty() {
            total_response_time / results.len() as u64
        } else {
            0
        };
        analytics.top_domains = domain_counts.into_iter().collect::<Vec<_>>();
        analytics.top_domains.sort_by(|a, b| b.1.cmp(&a.1));
        analytics.top_domains.truncate(10);

        analytics.language_distribution = language_counts;

        Ok(analytics)
    }

    /// Create storage with sensible defaults
    pub fn new_default() -> Result<Self> {
        Self::new("./crawl_data", OutputFormat::Jsonl)
    }

    /// Create storage in current directory with specified format
    pub fn with_format(format: OutputFormat) -> Result<Self> {
        Self::new("./crawl_data", format)
    }

    /// Helper method to generate filename
    fn generate_filename(&self, url: &str, timestamp: &SystemTime) -> String {
        let mut hasher = DefaultHasher::new();
        hasher.write(url.as_bytes());
        let url_hash = format!("{:x}", hasher.finish());
        let timestamp_secs = timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let extension = match &self.format {
            OutputFormat::Json => "json",
            OutputFormat::Jsonl => "jsonl",
            OutputFormat::Csv => "csv",
            OutputFormat::Parquet => "parquet",
        };

        format!("crawl_{}_{}.{}", timestamp_secs, &url_hash[..8], extension)
    }

    /// Write content to file
    async fn write_to_file(&self, path: &Path, content: String) -> Result<()> {
        if self.compression {
            // TODO: Implement compression
            fs::write(path, content).await?;
        } else {
            fs::write(path, content).await?;
        }
        Ok(())
    }

    /// Append content to file
    async fn append_to_file(&self, path: &Path, content: String) -> Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Store result as CSV
    async fn store_as_csv(&self, result: &StoredCrawlResult, path: &Path) -> Result<()> {
        // Implementation for CSV storage
        let csv_line = format!(
            "{},{},{},{},{},{}\n",
            result.url,
            result.title.as_deref().unwrap_or(""),
            result.word_count,
            result.language.as_deref().unwrap_or(""),
            result.metadata.response_time_ms,
            result.metadata.status_code.unwrap_or(0)
        );

        self.append_to_file(path, csv_line).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlSessionSummary {
    pub session_id: String,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub total_urls_processed: usize,
    pub successful_crawls: usize,
    pub failed_crawls: usize,
    pub total_bytes_downloaded: u64,
    pub unique_domains: usize,
    pub configuration: String, // Serialized config
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrawlAnalytics {
    pub total_pages: usize,
    pub successful_crawls: usize,
    pub failed_crawls: usize,
    pub domains_crawled: usize,
    pub avg_words_per_page: usize,
    pub avg_response_time_ms: u64,
    pub top_domains: Vec<(String, usize)>,
    pub language_distribution: HashMap<String, usize>,
}
