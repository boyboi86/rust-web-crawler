import { invoke } from '@tauri-apps/api/core';
import { CrawlRequest, CrawlStatus, WebCrawlerConfig, CrawlerFormConfig } from '../types/crawler';

// Debug function to test Tauri environment
export const debugTauriEnvironment = () => {
  console.log('🔍 Comprehensive Tauri Environment Debug:');
  console.log('Window object:', typeof window);

  if (typeof window !== 'undefined') {
    console.log('Location:', window.location);
    console.log('Protocol:', window.location.protocol);
    console.log('Host:', window.location.host);
    console.log('UserAgent:', window.navigator.userAgent);
    console.log('__TAURI__:', (window as any).__TAURI__);
    console.log('__TAURI_INVOKE__:', (window as any).__TAURI_INVOKE__);
    console.log('__TAURI_METADATA__:', (window as any).__TAURI_METADATA__);
  }

  console.log('invoke function:', typeof invoke);
  console.log(
    'invoke function string:',
    invoke ? invoke.toString().substring(0, 100) : 'undefined'
  );

  // Test if we can access Tauri APIs
  try {
    const testResult = typeof invoke === 'function';
    console.log('Can access invoke function:', testResult);
  } catch (error) {
    console.log('Error accessing invoke:', error);
  }

  return {
    hasWindow: typeof window !== 'undefined',
    protocol: typeof window !== 'undefined' ? window.location.protocol : null,
    hasTauri: typeof window !== 'undefined' && !!(window as any).__TAURI__,
    hasInvoke: typeof invoke === 'function',
    userAgent: typeof window !== 'undefined' ? window.navigator.userAgent : null,
  };
};

// Check if we're in a Tauri environment
const isTauriEnvironment = (): boolean => {
  // Check multiple ways Tauri can be detected
  const hasTauriGlobal = typeof window !== 'undefined' && (window as any).__TAURI__;
  const hasInvokeFunction = typeof invoke === 'function';
  const hasTauriAPI = typeof window !== 'undefined' && (window as any).__TAURI_INVOKE__;
  const isTauriApp =
    typeof window !== 'undefined' && (window as any).location?.protocol === 'tauri:';

  console.log('🔍 Tauri Environment Check:', {
    hasTauriGlobal,
    hasInvokeFunction,
    hasTauriAPI,
    isTauriApp,
    protocol: typeof window !== 'undefined' ? window.location?.protocol : 'undefined',
    userAgent: typeof window !== 'undefined' ? window.navigator?.userAgent : 'undefined',
  });

  return hasTauriGlobal || hasInvokeFunction || hasTauriAPI || isTauriApp;
};

// Test function that bypasses environment check
export const testTauriCommand = async (command: string, args?: any) => {
  console.log(`🧪 Testing Tauri command directly: ${command}`, args);

  try {
    if (typeof invoke !== 'function') {
      throw new Error('invoke function is not available');
    }

    const result = await invoke(command, args);
    console.log(`✅ Direct Tauri test successful:`, result);
    return result;
  } catch (error) {
    console.error(`❌ Direct Tauri test failed:`, error);
    throw error;
  }
};

// Safe invoke wrapper with better error handling and timeout
const safeInvoke = async <T>(
  command: string,
  args?: any,
  timeoutMs: number = 30000
): Promise<T> => {
  console.log(`🚀 Attempting to invoke Tauri command: ${command}`, args);

  // First, try to call invoke directly with timeout
  if (typeof invoke === 'function') {
    try {
      // Create a timeout promise
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => {
          reject(new Error(`Tauri command '${command}' timed out after ${timeoutMs}ms`));
        }, timeoutMs);
      });

      // Race between the invoke call and the timeout
      const result = await Promise.race([invoke<T>(command, args), timeoutPromise]);

      console.log(`✅ Tauri command ${command} succeeded:`, result);
      return result;
    } catch (error) {
      console.error(`❌ Tauri command ${command} failed:`, error);
      throw error;
    }
  }

  // If invoke is not available, check environment and provide detailed error
  if (!isTauriEnvironment()) {
    const errorMsg = `Tauri environment not detected. Make sure you're running this app through Tauri, not a regular browser. Command: ${command}`;
    console.error('❌', errorMsg);
    throw new Error(errorMsg);
  }

  // Final fallback error
  const errorMsg = `Tauri invoke function is not available. Command: ${command}`;
  console.error('❌', errorMsg);
  throw new Error(errorMsg);
}; // Generate unique session ID
export const generateSessionId = (): string => {
  return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
};

// Convert form config to crawler request
export const formConfigToCrawlRequest = (
  formConfig: CrawlerFormConfig,
  sessionId?: string
): CrawlRequest => {
  console.log('🔄 formConfigToCrawlRequest started with input:', formConfig);
  console.log('📝 Input form config keys:', Object.keys(formConfig));
  console.log('📝 Input form config values:', Object.values(formConfig));

  const request: CrawlRequest = {
    session_id: sessionId || generateSessionId(),
    base_url: formConfig.baseUrl,
    max_total_urls: formConfig.maxTotalUrls,
    max_crawl_depth: formConfig.maxCrawlDepth,
    enable_discovery_crawling: formConfig.enableDiscoveryCrawling,
    enable_keyword_filtering: formConfig.enableKeywordFiltering,
    target_words: formConfig.targetWords,
    enable_content_filtering: formConfig.enableContentFiltering,
    avoid_url_extensions: formConfig.avoidUrlExtensions,
    enable_language_filtering: formConfig.enableLanguageFiltering,
    latin_word_filter: formConfig.latinWordFilter,
    match_strategy: formConfig.matchStrategy,
    // Enhanced fields
    user_agent: formConfig.userAgent,
    proxy_pool: formConfig.proxyPool,
    accepted_languages: formConfig.acceptedLanguages,
    min_word_length: formConfig.minWordLength,
    default_rate_limit: formConfig.defaultRateLimit
      ? {
          max_requests_per_second: formConfig.defaultRateLimit.maxRequestsPerSecond,
          window_size_ms: formConfig.defaultRateLimit.windowSizeMs,
        }
      : undefined,
    retry_config: formConfig.retryConfig
      ? {
          max_retries: formConfig.retryConfig.maxRetries,
          base_delay_ms: formConfig.retryConfig.baseDelayMs,
          max_delay_ms: formConfig.retryConfig.maxDelayMs,
          backoff_multiplier: formConfig.retryConfig.backoffMultiplier,
          jitter_factor: formConfig.retryConfig.jitterFactor,
        }
      : undefined,
    logging_config: formConfig.loggingConfig
      ? {
          level: formConfig.loggingConfig.level,
          file_path: formConfig.loggingConfig.filePath,
          json_format: formConfig.loggingConfig.jsonFormat,
        }
      : undefined,
  };

  console.log('✅ formConfigToCrawlRequest generated CrawlRequest:', request);
  console.log('📝 Output request keys:', Object.keys(request));
  console.log('📝 Output request values:', Object.values(request));

  // Validate that key data wasn't lost
  if (formConfig.userAgent && !request.user_agent) {
    console.error('❌ DATA LOSS: userAgent was not transferred!', {
      input: formConfig.userAgent,
      output: request.user_agent,
    });
  }

  if (formConfig.proxyPool && !request.proxy_pool) {
    console.error('❌ DATA LOSS: proxyPool was not transferred!', {
      input: formConfig.proxyPool,
      output: request.proxy_pool,
    });
  }

  return request;
};

// Tauri API calls
export class CrawlerAPI {
  // Get default configuration from backend
  static async getDefaultConfig(): Promise<WebCrawlerConfig> {
    try {
      return await safeInvoke<WebCrawlerConfig>('get_default_config');
    } catch (error) {
      console.error('Failed to get default config:', error);
      throw new Error(`Failed to get default config: ${error}`);
    }
  }

  // Validate crawler configuration
  static async validateConfig(request: CrawlRequest): Promise<string> {
    try {
      console.log('🔍 Calling Tauri invoke: validate_crawl_request_api with request:', request);
      const result = await safeInvoke<string>('validate_crawl_request_api', { request });
      console.log('✅ validate_crawl_request_api returned:', result);
      return result;
    } catch (error) {
      console.error('❌ validate_crawl_request_api failed:', error);
      throw new Error(`Config validation failed: ${error}`);
    }
  }

  // Initialize a new crawl session
  static async startCrawl(request: CrawlRequest): Promise<string> {
    try {
      console.log('🚀 Calling Tauri invoke: start_crawl with request:', request);
      const result = await safeInvoke<string>('start_crawl', { request });
      console.log('✅ start_crawl returned:', result);
      return result;
    } catch (error) {
      console.error('❌ start_crawl failed:', error);
      throw new Error(`Failed to start crawl: ${error}`);
    }
  }

  // Remove executeCrawl method since it's not needed for now
  // The crawl session will be initialized and ready for future execution

  // Get crawl status
  static async getCrawlStatus(sessionId: string): Promise<CrawlStatus> {
    try {
      return await safeInvoke<CrawlStatus>('get_crawl_status', { sessionId });
    } catch (error) {
      console.error('Failed to get crawl status:', error);
      throw new Error(`Failed to get crawl status: ${error}`);
    }
  }

  // Stop crawl session
  static async stopCrawl(sessionId: string): Promise<string> {
    try {
      return await safeInvoke<string>('stop_crawl', { sessionId });
    } catch (error) {
      console.error('Failed to stop crawl:', error);
      throw new Error(`Failed to stop crawl: ${error}`);
    }
  }

  // Convenience method to start crawl session
  static async startAndExecuteCrawl(formConfig: CrawlerFormConfig): Promise<{
    sessionId: string;
    validationResult: string;
    startResult: string;
  }> {
    console.log('🔄 CrawlerAPI.startAndExecuteCrawl called with formConfig:', formConfig);
    console.log('📊 Form config details:');
    console.log('  - baseUrl:', formConfig.baseUrl);
    console.log('  - userAgent:', formConfig.userAgent);
    console.log('  - proxyPool:', formConfig.proxyPool);
    console.log('  - retryConfig:', formConfig.retryConfig);

    const request = formConfigToCrawlRequest(formConfig);
    console.log('📋 Generated CrawlRequest:', request);
    console.log('📊 Request details:');
    console.log('  - base_url:', request.base_url);
    console.log('  - user_agent:', request.user_agent);
    console.log('  - proxy_pool:', request.proxy_pool);
    console.log('  - retry_config:', request.retry_config);

    try {
      // Validate configuration
      console.log('🔍 Validating configuration...');
      const validationResult = await this.validateConfig(request);
      console.log('✅ Configuration validation result:', validationResult);

      // Start the crawl session
      console.log('🚀 Starting crawl session...');
      console.log('🚀 Calling Tauri invoke: start_crawl with request:', request);
      const startResult = await this.startCrawl(request);
      console.log('✅ Crawl session started:', startResult);

      const result = {
        sessionId: request.session_id,
        validationResult,
        startResult,
      };

      console.log('🎉 startAndExecuteCrawl completed successfully:', result);
      return result;
    } catch (error) {
      console.error('💥 startAndExecuteCrawl failed:', error);
      console.error('💥 Failed request was:', request);
      console.error('💥 Original form config was:', formConfig);
      throw error;
    }
  }

  // Poll for crawl status updates
  static async pollCrawlStatus(
    sessionId: string,
    onUpdate: (status: CrawlStatus) => void,
    intervalMs: number = 2000
  ): Promise<() => void> {
    const poll = async () => {
      try {
        const status = await this.getCrawlStatus(sessionId);
        onUpdate(status);

        // Continue polling if still running
        if (status.status === 'running') {
          setTimeout(poll, intervalMs);
        }
      } catch (error) {
        console.error('Error polling status:', error);
        onUpdate({
          session_id: sessionId,
          status: 'error',
          total_urls_processed: 0,
          successful_crawls: 0,
          failed_crawls: 0,
          errors: [`Polling error: ${error}`],
          results: [],
        });
      }
    };

    // Start polling
    poll();

    // Return stop function
    return () => {
      // Note: In a real implementation, you'd want to track the timeout ID
      // and clear it to properly stop polling
    };
  }
}
