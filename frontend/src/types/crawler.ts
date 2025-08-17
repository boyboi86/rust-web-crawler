// TypeScript types matching the Rust backend structures

export interface CrawlRequest {
  session_id: string;
  base_url: string;
  max_total_urls: number;
  max_crawl_depth: number;
  enable_discovery_crawling: boolean;
  enable_keyword_filtering: boolean;
  target_words: string[];
  enable_content_filtering: boolean;
  avoid_url_extensions: string[];
  enable_language_filtering: boolean;
  latin_word_filter: boolean;
  match_strategy: 'any' | 'all';
  // Enhanced fields to match WebCrawlerConfig
  user_agent?: string;
  proxy_pool?: string[];
  accepted_languages?: string[];
  min_word_length?: number;
  default_rate_limit?: {
    max_requests_per_second: number;
    window_size_ms: number;
  };
  retry_config?: {
    max_retries: number;
    base_delay_ms: number;
    max_delay_ms: number;
    backoff_multiplier: number;
    jitter_factor: number;
  };
  logging_config?: {
    level: string;
    file_path: string;
    json_format: boolean;
  };
}

export interface CrawlStatus {
  session_id: string;
  status: 'idle' | 'running' | 'completed' | 'error';
  total_urls_processed: number;
  successful_crawls: number;
  failed_crawls: number;
  current_url?: string;
  errors: string[];
  results: CrawlResultSummary[];
}

export interface CrawlResultSummary {
  url: string;
  title?: string;
  word_count: number;
  target_words_found: string[];
  language?: string;
  status_code?: number;
}

export interface WebCrawlerConfig {
  base_url: string[];
  max_total_urls: number;
  max_crawl_depth: number;
  enable_extension_crawling: boolean;
  enable_keyword_filtering: boolean;
  target_words: string[];
  avoid_url_extensions: string[];
  latin_word_filter: {
    exclude_numeric: boolean;
    excluded_words: string[];
    min_word_length: number;
  };
  // Enhanced fields to match the complete configuration
  user_agent: string;
  proxy_pool: string[];
  accepted_languages: string[];
  min_word_length: number;
  default_rate_limit: {
    max_requests_per_second: number;
    window_size_ms: number;
  };
  domain_rate_limits?: any;
  retry_config: {
    max_retries: number;
    base_delay_ms: number;
    max_delay_ms: number;
    backoff_multiplier: number;
    jitter_factor: number;
  };
  logging_config: {
    level: string;
    file_path: string;
    json_format: boolean;
  };
}

// Form configuration state that matches the UI
export interface CrawlerFormConfig {
  baseUrl: string;
  maxTotalUrls: number;
  maxCrawlDepth: number;
  enableDiscoveryCrawling: boolean;
  enableKeywordFiltering: boolean;
  targetWords: string[];
  enableContentFiltering: boolean;
  avoidUrlExtensions: string[];
  enableLanguageFiltering: boolean;
  latinWordFilter: boolean;
  matchStrategy: 'any' | 'all';
  // Enhanced fields
  userAgent?: string;
  proxyPool?: string[];
  acceptedLanguages?: string[];
  minWordLength?: number;
  defaultRateLimit?: {
    maxRequestsPerSecond: number;
    windowSizeMs: number;
  };
  retryConfig?: {
    maxRetries: number;
    baseDelayMs: number;
    maxDelayMs: number;
    backoffMultiplier: number;
    jitterFactor: number;
  };
  loggingConfig?: {
    level: string;
    filePath: string;
    jsonFormat: boolean;
  };
}
