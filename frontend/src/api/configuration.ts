// =============================================================================
// CONFIGURATION API CLIENT
// =============================================================================
// TypeScript interface for consuming backend configuration from the single
// source of truth. This ensures the frontend never has hardcoded values.

import { invoke } from '@tauri-apps/api/core';

// =============================================================================
// CONFIGURATION TYPES (matching Rust backend)
// =============================================================================

export interface AppConfig {
  network: NetworkConfig;
  crawling: CrawlingConfig;
  rate_limiting: RateLimitingConfig;
  retry: RetryConfig;
  logging: LoggingConfig;
  proxy: ProxyConfig;
  content: ContentConfig;
  frontend: FrontendConfig;
  development: DevelopmentConfig;
}

export interface NetworkConfig {
  user_agent: string;
  request_timeout_secs: number;
  max_redirects: number;
  politeness_delay_ms: number;
}

export interface CrawlingConfig {
  max_total_urls: number;
  max_crawl_depth: number;
  min_word_length: number;
  max_concurrent_requests: number;
  avoid_extensions: string[];
  target_words: string[];
  excluded_words: string[];
}

export interface RateLimitingConfig {
  requests_per_second: number;
  window_ms: number;
}

export interface RetryConfig {
  max_retries: number;
  base_delay_ms: number;
  max_delay_ms: number;
  backoff_multiplier: number;
  jitter_factor: number;
}

export interface LoggingConfig {
  level: string;
  file_path: string;
  json_format: boolean;
}

export interface ProxyConfig {
  proxy_pool: string[];
  timeout_secs: number;
}

export interface ContentConfig {
  min_content_length: number;
  language_content_percentage: number;
  accepted_languages: string[];
}

export interface FrontendConfig {
  status_poll_interval_ms: number;
  form_validation_timeout_ms: number;
}

export interface DevelopmentConfig {
  debug_mode: boolean;
  enable_metrics: boolean;
  metrics_port: number;
  health_check_interval_secs: number;
}

export interface FrontendConfigResponse {
  app_config: AppConfig;
  presets: FrontendPresets;
  environment_info: EnvironmentInfo;
}

export interface FrontendPresets {
  yahoo: AppConfig;
  test: AppConfig;
  development: AppConfig;
}

export interface EnvironmentInfo {
  is_debug: boolean;
  version: string;
  platform: string;
  config_source: string;
}

// =============================================================================
// CONFIGURATION API CLIENT
// =============================================================================

export class ConfigurationClient {
  private static _instance: ConfigurationClient;
  private _config: FrontendConfigResponse | null = null;
  private _loadPromise: Promise<FrontendConfigResponse> | null = null;

  private constructor() {}

  public static getInstance(): ConfigurationClient {
    if (!ConfigurationClient._instance) {
      ConfigurationClient._instance = new ConfigurationClient();
    }
    return ConfigurationClient._instance;
  }

  /**
   * Load configuration from backend (single source of truth)
   * This should be called once during app initialization
   */
  public async loadConfiguration(): Promise<FrontendConfigResponse> {
    if (this._config) {
      return this._config;
    }

    if (this._loadPromise) {
      return this._loadPromise;
    }

    this._loadPromise = this._fetchConfiguration();
    this._config = await this._loadPromise;
    this._loadPromise = null;

    console.log('✅ Configuration loaded from backend:', this._config);
    return this._config;
  }

  private async _fetchConfiguration(): Promise<FrontendConfigResponse> {
    try {
      const config = await invoke<FrontendConfigResponse>('get_app_config');
      console.log('📡 Configuration received from backend:', config);
      return config;
    } catch (error) {
      console.error('❌ Failed to load configuration from backend:', error);
      throw new Error(`Failed to load configuration: ${error}`);
    }
  }

  /**
   * Get the current configuration (must call loadConfiguration first)
   */
  public getConfig(): FrontendConfigResponse {
    if (!this._config) {
      throw new Error('Configuration not loaded. Call loadConfiguration() first.');
    }
    return this._config;
  }

  /**
   * Get a specific preset configuration
   */
  public async getPreset(presetName: string): Promise<AppConfig> {
    try {
      return await invoke<AppConfig>('get_config_preset', { presetName });
    } catch (error) {
      console.error(`❌ Failed to get preset ${presetName}:`, error);
      throw error;
    }
  }

  /**
   * Validate a configuration object
   */
  public async validateConfig(config: AppConfig): Promise<boolean> {
    try {
      return await invoke<boolean>('validate_config', { config });
    } catch (error) {
      console.error('❌ Configuration validation failed:', error);
      throw error;
    }
  }

  /**
   * Get configuration summary for debugging
   */
  public async getConfigSummary(): Promise<string> {
    try {
      return await invoke<string>('get_config_summary');
    } catch (error) {
      console.error('❌ Failed to get config summary:', error);
      throw error;
    }
  }

  /**
   * Get environment variables documentation
   */
  public async getEnvDocumentation(): Promise<string> {
    try {
      return await invoke<string>('get_env_documentation');
    } catch (error) {
      console.error('❌ Failed to get env documentation:', error);
      throw error;
    }
  }

  /**
   * Force reload configuration from backend
   */
  public async reloadConfiguration(): Promise<FrontendConfigResponse> {
    this._config = null;
    this._loadPromise = null;
    return this.loadConfiguration();
  }

  // =============================================================================
  // CONVENIENCE METHODS
  // =============================================================================

  /**
   * Get default user agent from configuration
   */
  public getDefaultUserAgent(): string {
    const config = this.getConfig();
    return config.app_config.network.user_agent;
  }

  /**
   * Get default form values from configuration
   */
  public getDefaultFormValues() {
    const config = this.getConfig();
    return {
      userAgent: config.app_config.network.user_agent,
      maxUrls: config.app_config.crawling.max_total_urls,
      maxDepth: config.app_config.crawling.max_crawl_depth,
      requestTimeout: config.app_config.network.request_timeout_secs,
      rateLimit: config.app_config.rate_limiting.requests_per_second,
      minWordLength: config.app_config.crawling.min_word_length,
      acceptedLanguages: config.app_config.content.accepted_languages,
      proxyPool: config.app_config.proxy.proxy_pool,
    };
  }

  /**
   * Get Yahoo-specific configuration
   */
  public getYahooConfig(): AppConfig {
    const config = this.getConfig();
    return config.presets.yahoo;
  }

  /**
   * Get development configuration
   */
  public getDevConfig(): AppConfig {
    const config = this.getConfig();
    return config.presets.development;
  }

  /**
   * Get test configuration
   */
  public getTestConfig(): AppConfig {
    const config = this.getConfig();
    return config.presets.test;
  }

  /**
   * Check if we're in debug mode
   */
  public isDebugMode(): boolean {
    const config = this.getConfig();
    return config.environment_info.is_debug;
  }

  /**
   * Get frontend-specific configuration
   */
  public getFrontendConfig(): FrontendConfig {
    const config = this.getConfig();
    return config.app_config.frontend;
  }
}

// =============================================================================
// CONVENIENT EXPORTS
// =============================================================================

/**
 * Global configuration client instance
 */
export const configClient = ConfigurationClient.getInstance();

/**
 * Initialize configuration on app startup
 * This should be called in your main App component
 */
export async function initializeConfiguration(): Promise<FrontendConfigResponse> {
  console.log('🔧 Initializing configuration from backend...');
  const config = await configClient.loadConfiguration();
  console.log('✅ Configuration initialized successfully');
  console.log(`📊 Config source: ${config.environment_info.config_source}`);
  console.log(`🏷️  App version: ${config.environment_info.version}`);
  console.log(`💻 Platform: ${config.environment_info.platform}`);
  console.log(`🐛 Debug mode: ${config.environment_info.is_debug}`);
  return config;
}

/**
 * Get form defaults without needing to instantiate the client
 */
export async function getFormDefaults() {
  await configClient.loadConfiguration();
  return configClient.getDefaultFormValues();
}

/**
 * Helper to create a crawl request with proper defaults
 */
export async function createCrawlRequestWithDefaults(
  baseUrl: string,
  overrides: Partial<any> = {}
) {
  await configClient.loadConfiguration();
  const defaults = configClient.getDefaultFormValues();

  return {
    session_id: `crawl_${Date.now()}`,
    base_url: baseUrl,
    user_agent: overrides.userAgent || defaults.userAgent,
    max_total_urls: overrides.maxUrls || defaults.maxUrls,
    max_crawl_depth: overrides.maxDepth || defaults.maxDepth,
    min_word_length: overrides.minWordLength || defaults.minWordLength,
    accepted_languages: overrides.acceptedLanguages || defaults.acceptedLanguages,
    proxy_pool: overrides.proxyPool || defaults.proxyPool,
    default_rate_limit: overrides.rateLimit || defaults.rateLimit,
    // Add any other fields based on your CrawlRequest type
  };
}
