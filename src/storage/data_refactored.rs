/// Data storage module - refactored using common building blocks
/// Following Rule 1: No hardcoding - all configuration external
/// Following Rule 3: Builder pattern for complex storage configurations
/// Following Rule 4: Privacy first - controlled access to storage operations
/// Following Rule 8: Idiomatic Rust - Result<T,E>, functional patterns
use crate::common::{
    BooleanFlag, ConfigResult, LimitValue, ProcessingResult, SessionId, TaskContent, TaskError,
    TaskId, TaskResult, UrlString,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Supported storage formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageFormat {
    Json,
    JsonLines,
    Csv,
    Txt,
}

impl Default for StorageFormat {
    fn default() -> Self {
        StorageFormat::Json
    }
}

impl StorageFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            StorageFormat::Json => "json",
            StorageFormat::JsonLines => "jsonl",
            StorageFormat::Csv => "csv",
            StorageFormat::Txt => "txt",
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            StorageFormat::Json => "application/json",
            StorageFormat::JsonLines => "application/x-ndjson",
            StorageFormat::Csv => "text/csv",
            StorageFormat::Txt => "text/plain",
        }
    }
}

/// Storage compression options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::None
    }
}

impl CompressionType {
    pub fn file_extension(&self) -> Option<&'static str> {
        match self {
            CompressionType::None => None,
            CompressionType::Gzip => Some("gz"),
            CompressionType::Zstd => Some("zst"),
        }
    }
}

/// Configuration for data storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    // Basic storage settings
    output_directory: PathBuf,
    storage_format: StorageFormat,
    compression: CompressionType,

    // File management
    max_file_size: LimitValue,
    rotate_files: BooleanFlag,
    files_per_session: LimitValue,
    create_directories: BooleanFlag,

    // Content settings
    include_content: BooleanFlag,
    include_metadata: BooleanFlag,
    include_links: BooleanFlag,
    pretty_print: BooleanFlag,

    // Performance settings
    buffer_size: LimitValue,
    flush_frequency: LimitValue,

    // Session tracking
    session_id: SessionId,
    use_session_subdirectory: BooleanFlag,
}

impl StorageConfig {
    pub fn builder() -> StorageConfigBuilder {
        StorageConfigBuilder::new()
    }

    pub fn output_directory(&self) -> &Path {
        &self.output_directory
    }

    pub fn storage_format(&self) -> StorageFormat {
        self.storage_format
    }

    pub fn compression(&self) -> CompressionType {
        self.compression
    }

    pub fn max_file_size(&self) -> u64 {
        self.max_file_size.value()
    }

    pub fn should_rotate_files(&self) -> bool {
        self.rotate_files.is_enabled()
    }

    pub fn files_per_session(&self) -> u64 {
        self.files_per_session.value()
    }

    pub fn should_create_directories(&self) -> bool {
        self.create_directories.is_enabled()
    }

    pub fn should_include_content(&self) -> bool {
        self.include_content.is_enabled()
    }

    pub fn should_include_metadata(&self) -> bool {
        self.include_metadata.is_enabled()
    }

    pub fn should_include_links(&self) -> bool {
        self.include_links.is_enabled()
    }

    pub fn should_pretty_print(&self) -> bool {
        self.pretty_print.is_enabled()
    }

    pub fn buffer_size(&self) -> u64 {
        self.buffer_size.value()
    }

    pub fn flush_frequency(&self) -> u64 {
        self.flush_frequency.value()
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn should_use_session_subdirectory(&self) -> bool {
        self.use_session_subdirectory.is_enabled()
    }

    /// Get the complete file path with extensions
    pub fn get_file_path(&self, base_name: &str) -> PathBuf {
        let mut path = self.output_directory.clone();

        if self.should_use_session_subdirectory() {
            path = path.join(self.session_id.as_str());
        }

        let mut filename = format!("{}.{}", base_name, self.storage_format.file_extension());

        if let Some(compression_ext) = self.compression.file_extension() {
            filename = format!("{}.{}", filename, compression_ext);
        }

        path.join(filename)
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::from("./crawl_data"),
            storage_format: StorageFormat::Json,
            compression: CompressionType::None,
            max_file_size: LimitValue::new(100 * 1024 * 1024), // 100MB
            rotate_files: BooleanFlag::enabled(),
            files_per_session: LimitValue::new(10),
            create_directories: BooleanFlag::enabled(),
            include_content: BooleanFlag::enabled(),
            include_metadata: BooleanFlag::enabled(),
            include_links: BooleanFlag::enabled(),
            pretty_print: BooleanFlag::enabled(),
            buffer_size: LimitValue::new(8192),    // 8KB
            flush_frequency: LimitValue::new(100), // Every 100 records
            session_id: SessionId::new(format!("storage_{}", uuid::Uuid::new_v4())),
            use_session_subdirectory: BooleanFlag::enabled(),
        }
    }
}

/// Builder for storage configuration
#[derive(Debug)]
pub struct StorageConfigBuilder {
    output_directory: PathBuf,
    storage_format: StorageFormat,
    compression: CompressionType,
    max_file_size: LimitValue,
    rotate_files: BooleanFlag,
    files_per_session: LimitValue,
    create_directories: BooleanFlag,
    include_content: BooleanFlag,
    include_metadata: BooleanFlag,
    include_links: BooleanFlag,
    pretty_print: BooleanFlag,
    buffer_size: LimitValue,
    flush_frequency: LimitValue,
    session_id: SessionId,
    use_session_subdirectory: BooleanFlag,
}

impl StorageConfigBuilder {
    pub fn new() -> Self {
        let default_config = StorageConfig::default();
        Self {
            output_directory: default_config.output_directory,
            storage_format: default_config.storage_format,
            compression: default_config.compression,
            max_file_size: default_config.max_file_size,
            rotate_files: default_config.rotate_files,
            files_per_session: default_config.files_per_session,
            create_directories: default_config.create_directories,
            include_content: default_config.include_content,
            include_metadata: default_config.include_metadata,
            include_links: default_config.include_links,
            pretty_print: default_config.pretty_print,
            buffer_size: default_config.buffer_size,
            flush_frequency: default_config.flush_frequency,
            session_id: default_config.session_id,
            use_session_subdirectory: default_config.use_session_subdirectory,
        }
    }

    pub fn with_output_directory(mut self, directory: PathBuf) -> Self {
        self.output_directory = directory;
        self
    }

    pub fn with_storage_format(mut self, format: StorageFormat) -> Self {
        self.storage_format = format;
        self
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }

    pub fn with_file_management(
        mut self,
        max_size: LimitValue,
        rotate: BooleanFlag,
        files_per_session: LimitValue,
    ) -> Self {
        self.max_file_size = max_size;
        self.rotate_files = rotate;
        self.files_per_session = files_per_session;
        self
    }

    pub fn with_content_options(
        mut self,
        include_content: BooleanFlag,
        include_metadata: BooleanFlag,
        include_links: BooleanFlag,
        pretty_print: BooleanFlag,
    ) -> Self {
        self.include_content = include_content;
        self.include_metadata = include_metadata;
        self.include_links = include_links;
        self.pretty_print = pretty_print;
        self
    }

    pub fn with_performance_settings(
        mut self,
        buffer_size: LimitValue,
        flush_frequency: LimitValue,
    ) -> Self {
        self.buffer_size = buffer_size;
        self.flush_frequency = flush_frequency;
        self
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_session_subdirectory(mut self, enabled: BooleanFlag) -> Self {
        self.use_session_subdirectory = enabled;
        self
    }

    pub fn build(self) -> StorageConfig {
        StorageConfig {
            output_directory: self.output_directory,
            storage_format: self.storage_format,
            compression: self.compression,
            max_file_size: self.max_file_size,
            rotate_files: self.rotate_files,
            files_per_session: self.files_per_session,
            create_directories: self.create_directories,
            include_content: self.include_content,
            include_metadata: self.include_metadata,
            include_links: self.include_links,
            pretty_print: self.pretty_print,
            buffer_size: self.buffer_size,
            flush_frequency: self.flush_frequency,
            session_id: self.session_id,
            use_session_subdirectory: self.use_session_subdirectory,
        }
    }
}

impl Default for StorageConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Stored crawl result data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCrawlResult {
    // Task identification
    task_id: TaskId,
    url: UrlString,
    timestamp: SystemTime,

    // Content data (optional based on configuration)
    content: Option<String>,
    content_type: Option<String>,
    content_length: u64,

    // Extracted metadata (optional based on configuration)
    metadata: HashMap<String, String>,

    // Links found (optional based on configuration)
    links: Vec<UrlString>,

    // Processing information
    processing_time_ms: u64,
    success: bool,
    error_message: Option<String>,
}

impl StoredCrawlResult {
    pub fn from_task_result(task_result: &TaskResult<TaskContent>, config: &StorageConfig) -> Self {
        let content_data = if task_result.is_success() {
            task_result.data()
        } else {
            None
        };

        let (content, content_type, metadata, links) = if let Some(task_content) = content_data {
            let content = if config.should_include_content() {
                Some(task_content.text().to_string())
            } else {
                None
            };

            let content_type = Some(task_content.content_type().to_string());

            let metadata = if config.should_include_metadata() {
                task_content.metadata().clone()
            } else {
                HashMap::new()
            };

            let links = if config.should_include_links() {
                task_content.links().to_vec()
            } else {
                Vec::new()
            };

            (content, content_type, metadata, links)
        } else {
            (None, None, HashMap::new(), Vec::new())
        };

        Self {
            task_id: task_result.task_id().clone(),
            url: task_result.url().clone(),
            timestamp: SystemTime::now(),
            content,
            content_type,
            content_length: content_data.map(|c| c.content_length()).unwrap_or(0),
            metadata,
            links,
            processing_time_ms: task_result.timing().total_duration().as_millis() as u64,
            success: task_result.is_success(),
            error_message: task_result.error().map(|e| e.to_string()),
        }
    }

    // Getters for all fields
    pub fn task_id(&self) -> &TaskId {
        &self.task_id
    }
    pub fn url(&self) -> &UrlString {
        &self.url
    }
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
    pub fn content_length(&self) -> u64 {
        self.content_length
    }
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    pub fn links(&self) -> &[UrlString] {
        &self.links
    }
    pub fn processing_time_ms(&self) -> u64 {
        self.processing_time_ms
    }
    pub fn success(&self) -> bool {
        self.success
    }
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

/// Data storage manager using building blocks
/// Following Rule 4: Privacy first - storage operations encapsulated
pub struct DataStorage {
    // Private configuration and state
    config: StorageConfig,
    current_file_index: std::sync::atomic::AtomicUsize,
    write_buffer: tokio::sync::Mutex<Vec<StoredCrawlResult>>,
    records_written: std::sync::atomic::AtomicUsize,
}

impl DataStorage {
    pub fn new(config: StorageConfig) -> Self {
        Self {
            config,
            current_file_index: std::sync::atomic::AtomicUsize::new(0),
            write_buffer: tokio::sync::Mutex::new(
                Vec::with_capacity(config.buffer_size() as usize),
            ),
            records_written: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(StorageConfig::default())
    }

    /// Store a single task result
    pub async fn store_result(
        &self,
        task_result: &TaskResult<TaskContent>,
    ) -> ProcessingResult<()> {
        let stored_result = StoredCrawlResult::from_task_result(task_result, &self.config);

        // Add to buffer
        {
            let mut buffer = self.write_buffer.lock().await;
            buffer.push(stored_result);

            // Check if we need to flush
            if buffer.len() >= self.config.flush_frequency() as usize {
                self.flush_buffer_internal(&mut buffer).await?;
            }
        }

        Ok(())
    }

    /// Store multiple task results
    pub async fn store_results(
        &self,
        task_results: &[TaskResult<TaskContent>],
    ) -> ProcessingResult<()> {
        for result in task_results {
            self.store_result(result).await?;
        }
        Ok(())
    }

    /// Flush buffered data to storage
    pub async fn flush(&self) -> ProcessingResult<()> {
        let mut buffer = self.write_buffer.lock().await;
        if !buffer.is_empty() {
            self.flush_buffer_internal(&mut buffer).await?;
        }
        Ok(())
    }

    /// Internal flush implementation
    async fn flush_buffer_internal(
        &self,
        buffer: &mut Vec<StoredCrawlResult>,
    ) -> ProcessingResult<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        // Create output directory if needed
        if self.config.should_create_directories() {
            self.ensure_directory_exists().await?;
        }

        // Determine file path
        let file_path = self.get_current_file_path().await?;

        // Write data based on format
        match self.config.storage_format() {
            StorageFormat::Json => self.write_json(&file_path, buffer).await?,
            StorageFormat::JsonLines => self.write_jsonlines(&file_path, buffer).await?,
            StorageFormat::Csv => self.write_csv(&file_path, buffer).await?,
            StorageFormat::Txt => self.write_txt(&file_path, buffer).await?,
        }

        // Update counters
        let written_count = buffer.len();
        self.records_written
            .fetch_add(written_count, std::sync::atomic::Ordering::Relaxed);

        // Clear buffer
        buffer.clear();

        Ok(())
    }

    /// Ensure output directory exists
    async fn ensure_directory_exists(&self) -> ProcessingResult<()> {
        let dir_path = if self.config.should_use_session_subdirectory() {
            self.config
                .output_directory()
                .join(self.config.session_id().as_str())
        } else {
            self.config.output_directory().to_path_buf()
        };

        fs::create_dir_all(&dir_path).await.map_err(|e| {
            TaskError::storage(format!("Failed to create directory {:?}: {}", dir_path, e))
        })?;

        Ok(())
    }

    /// Get current file path for writing
    async fn get_current_file_path(&self) -> ProcessingResult<PathBuf> {
        let base_name = if self.config.should_rotate_files() {
            let index = self
                .current_file_index
                .load(std::sync::atomic::Ordering::Relaxed);
            format!("crawl_results_{:04}", index)
        } else {
            "crawl_results".to_string()
        };

        let file_path = self.config.get_file_path(&base_name);

        // Check if file rotation is needed
        if self.config.should_rotate_files() && self.should_rotate_file(&file_path).await? {
            self.current_file_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let new_index = self
                .current_file_index
                .load(std::sync::atomic::Ordering::Relaxed);
            let new_base_name = format!("crawl_results_{:04}", new_index);
            return Ok(self.config.get_file_path(&new_base_name));
        }

        Ok(file_path)
    }

    /// Check if file should be rotated
    async fn should_rotate_file(&self, file_path: &Path) -> ProcessingResult<bool> {
        match fs::metadata(file_path).await {
            Ok(metadata) => Ok(metadata.len() >= self.config.max_file_size()),
            Err(_) => Ok(false), // File doesn't exist, no need to rotate
        }
    }

    /// Write data as JSON
    async fn write_json(
        &self,
        file_path: &Path,
        buffer: &[StoredCrawlResult],
    ) -> ProcessingResult<()> {
        let json_data = if self.config.should_pretty_print() {
            serde_json::to_string_pretty(buffer)
        } else {
            serde_json::to_string(buffer)
        }
        .map_err(|e| TaskError::storage(format!("JSON serialization failed: {}", e)))?;

        self.write_to_file(file_path, json_data.as_bytes()).await
    }

    /// Write data as JSON Lines
    async fn write_jsonlines(
        &self,
        file_path: &Path,
        buffer: &[StoredCrawlResult],
    ) -> ProcessingResult<()> {
        let mut jsonl_data = String::new();

        for result in buffer {
            let line = serde_json::to_string(result)
                .map_err(|e| TaskError::storage(format!("JSON serialization failed: {}", e)))?;
            jsonl_data.push_str(&line);
            jsonl_data.push('\n');
        }

        self.write_to_file(file_path, jsonl_data.as_bytes()).await
    }

    /// Write data as CSV
    async fn write_csv(
        &self,
        file_path: &Path,
        buffer: &[StoredCrawlResult],
    ) -> ProcessingResult<()> {
        let mut csv_data = String::new();

        // Write header if file doesn't exist
        if !file_path.exists() {
            csv_data.push_str("task_id,url,timestamp,content_type,content_length,processing_time_ms,success,error_message\n");
        }

        // Write data rows
        for result in buffer {
            csv_data.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                result.task_id().as_str(),
                result.url().as_str(),
                result
                    .timestamp()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                result.content_type().unwrap_or(""),
                result.content_length(),
                result.processing_time_ms(),
                result.success(),
                result.error_message().unwrap_or("")
            ));
        }

        self.write_to_file(file_path, csv_data.as_bytes()).await
    }

    /// Write data as plain text
    async fn write_txt(
        &self,
        file_path: &Path,
        buffer: &[StoredCrawlResult],
    ) -> ProcessingResult<()> {
        let mut txt_data = String::new();

        for result in buffer {
            txt_data.push_str(&format!(
                "URL: {}\nSuccess: {}\nContent Length: {}\nProcessing Time: {}ms\n",
                result.url().as_str(),
                result.success(),
                result.content_length(),
                result.processing_time_ms()
            ));

            if let Some(error) = result.error_message() {
                txt_data.push_str(&format!("Error: {}\n", error));
            }

            txt_data.push_str("---\n");
        }

        self.write_to_file(file_path, txt_data.as_bytes()).await
    }

    /// Write data to file
    async fn write_to_file(&self, file_path: &Path, data: &[u8]) -> ProcessingResult<()> {
        use tokio::fs::OpenOptions;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await
            .map_err(|e| {
                TaskError::storage(format!("Failed to open file {:?}: {}", file_path, e))
            })?;

        file.write_all(data).await.map_err(|e| {
            TaskError::storage(format!("Failed to write to file {:?}: {}", file_path, e))
        })?;

        file.flush().await.map_err(|e| {
            TaskError::storage(format!("Failed to flush file {:?}: {}", file_path, e))
        })?;

        Ok(())
    }

    /// Get storage statistics
    pub fn statistics(&self) -> StorageStatistics {
        let records_written = self
            .records_written
            .load(std::sync::atomic::Ordering::Relaxed);
        let current_file_index = self
            .current_file_index
            .load(std::sync::atomic::Ordering::Relaxed);

        StorageStatistics {
            records_written: records_written as u64,
            files_created: (current_file_index + 1) as u64,
            session_id: self.config.session_id().clone(),
            storage_format: self.config.storage_format(),
            compression: self.config.compression(),
        }
    }

    /// Get storage configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    /// Close storage and flush any remaining data
    pub async fn close(&self) -> ProcessingResult<()> {
        self.flush().await
    }
}

impl Default for DataStorage {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Storage operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    records_written: u64,
    files_created: u64,
    session_id: SessionId,
    storage_format: StorageFormat,
    compression: CompressionType,
}

impl StorageStatistics {
    pub fn records_written(&self) -> u64 {
        self.records_written
    }
    pub fn files_created(&self) -> u64 {
        self.files_created
    }
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn storage_format(&self) -> StorageFormat {
        self.storage_format
    }
    pub fn compression(&self) -> CompressionType {
        self.compression
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_storage_config_builder() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let config = StorageConfig::builder()
            .with_output_directory(temp_dir.path().to_path_buf())
            .with_storage_format(StorageFormat::JsonLines)
            .with_compression(CompressionType::Gzip)
            .build();

        assert_eq!(config.storage_format(), StorageFormat::JsonLines);
        assert_eq!(config.compression(), CompressionType::Gzip);
    }

    #[tokio::test]
    async fn test_data_storage_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let config = StorageConfig::builder()
            .with_output_directory(temp_dir.path().to_path_buf())
            .build();

        let storage = DataStorage::new(config);
        assert_eq!(storage.statistics().records_written(), 0);
    }

    #[tokio::test]
    async fn test_file_path_generation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let config = StorageConfig::builder()
            .with_output_directory(temp_dir.path().to_path_buf())
            .with_storage_format(StorageFormat::Json)
            .with_compression(CompressionType::Gzip)
            .build();

        let file_path = config.get_file_path("test");
        assert!(file_path.to_string_lossy().ends_with("test.json.gz"));
    }

    #[tokio::test]
    async fn test_storage_format_extensions() {
        assert_eq!(StorageFormat::Json.file_extension(), "json");
        assert_eq!(StorageFormat::JsonLines.file_extension(), "jsonl");
        assert_eq!(StorageFormat::Csv.file_extension(), "csv");
        assert_eq!(StorageFormat::Txt.file_extension(), "txt");
    }

    #[tokio::test]
    async fn test_compression_extensions() {
        assert_eq!(CompressionType::None.file_extension(), None);
        assert_eq!(CompressionType::Gzip.file_extension(), Some("gz"));
        assert_eq!(CompressionType::Zstd.file_extension(), Some("zst"));
    }
}
