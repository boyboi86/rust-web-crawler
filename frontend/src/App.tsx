import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { SettingsSidebar } from './components/layout'
import { CrawlerDashboard } from './components/features'
import { initializeConfiguration, configClient } from './api/configuration'

// Types based on the Rust code
export interface WebCrawlerConfig {
  accepted_languages: string[]
  target_words: string[]
  min_word_length: number
  min_content_length: number
  keyword_match_all: boolean
  language_content_percentage: number
  user_agent: string
  max_crawl_depth: number
  max_total_urls: number
  enable_extension_crawling: boolean
  enable_keyword_filtering: boolean
  avoid_url_extensions: string[]
  latin_word_filter: LatinWordFilter
  // Advanced configuration from TOML
  proxy_pool: string[]
  max_requests_per_second: number
  window_size_ms: number
  max_retries: number
  base_delay_ms: number
  max_delay_ms: number
  backoff_multiplier: number
  jitter_factor: number
  max_file_size_mb: number
  data_output_extension: 'csv' | 'json'
  enable_metrics: boolean
  metrics_port: number
  health_check_interval_secs: number
  persist_queue: boolean
  checkpoint_interval_secs: number
}

export interface LatinWordFilter {
  exclude_numeric: boolean
  excluded_words: string[]
  min_word_length: number
}

export interface CrawlResult {
  url: string
  title: string
  content: string
  status_code: number
  timestamp: string
}

export interface CrawlStatus {
  total_pages: number
  processed_pages: number
  queue_size: number
  is_running: boolean
}

export interface PresetConfig {
  name: string
  description: string
  config: WebCrawlerConfig
}

function App() {
  const [crawlStatus, setCrawlStatus] = useState<CrawlStatus | null>(null)
  const [config, setConfig] = useState<WebCrawlerConfig | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSidebarOpen, setIsSidebarOpen] = useState(false)

  // Load default configuration on startup
  useEffect(() => {
    loadDefaultConfig()
  }, [])

  // Poll for status updates when crawling is active
  useEffect(() => {
    let interval: number | null = null
    
    if (crawlStatus?.is_running) {
      interval = setInterval(async () => {
        try {
          const status: CrawlStatus = await invoke('get_crawl_status')
          setCrawlStatus(status)
        } catch (error) {
          // In development mode, simulate progress
          setCrawlStatus(prev => {
            if (!prev || !prev.is_running) return prev
            
            const newProcessed = Math.min(prev.processed_pages + 1, prev.total_pages)
            const newQueue = Math.max(prev.queue_size - 1, 0)
            
            return {
              ...prev,
              processed_pages: newProcessed,
              queue_size: newQueue,
              is_running: newProcessed < prev.total_pages && newQueue > 0
            }
          })
        }
      }, 1000)
    }

    return () => {
      if (interval) clearInterval(interval)
    }
  }, [crawlStatus?.is_running])

  const loadDefaultConfig = async () => {
    try {
      console.log('🔧 Loading configuration from backend...')
      
      // Use the new configuration system to get config from backend
      await initializeConfiguration()
      const backendConfig = configClient.getConfig()
      
      // Map backend config to frontend config format
      const mappedConfig: WebCrawlerConfig = {
        accepted_languages: backendConfig.app_config.content.accepted_languages,
        target_words: backendConfig.app_config.crawling.target_words,
        min_word_length: backendConfig.app_config.crawling.min_word_length,
        min_content_length: backendConfig.app_config.content.min_content_length,
        keyword_match_all: false, // Default for now
        language_content_percentage: backendConfig.app_config.content.language_content_percentage,
        user_agent: backendConfig.app_config.network.user_agent,
        max_crawl_depth: backendConfig.app_config.crawling.max_crawl_depth,
        max_total_urls: backendConfig.app_config.crawling.max_total_urls,
        enable_extension_crawling: false, // Default for now
        enable_keyword_filtering: false, // Default for now
        avoid_url_extensions: backendConfig.app_config.crawling.avoid_extensions,
        latin_word_filter: {
          exclude_numeric: true,
          excluded_words: backendConfig.app_config.crawling.excluded_words,
          min_word_length: backendConfig.app_config.crawling.min_word_length,
        },
        proxy_pool: backendConfig.app_config.proxy.proxy_pool,
        max_requests_per_second: backendConfig.app_config.rate_limiting.requests_per_second,
        window_size_ms: backendConfig.app_config.rate_limiting.window_ms,
        max_retries: backendConfig.app_config.retry.max_retries,
        base_delay_ms: backendConfig.app_config.retry.base_delay_ms,
        max_delay_ms: backendConfig.app_config.retry.max_delay_ms,
        backoff_multiplier: backendConfig.app_config.retry.backoff_multiplier,
        jitter_factor: backendConfig.app_config.retry.jitter_factor,
        max_file_size_mb: 10, // Default for now
        data_output_extension: 'json' as const,
        enable_metrics: backendConfig.app_config.development.enable_metrics,
        metrics_port: backendConfig.app_config.development.metrics_port,
        health_check_interval_secs: backendConfig.app_config.development.health_check_interval_secs,
        persist_queue: true, // Default for now
        checkpoint_interval_secs: 60, // Default for now
      }
      
      console.log('✅ Backend configuration loaded and mapped:', mappedConfig)
      console.log('📊 Configuration source:', backendConfig.environment_info.config_source)
      setConfig(mappedConfig)
      
    } catch (error) {
      console.error('❌ Failed to load config from backend:', error)
      
      // Fallback to legacy method for backward compatibility
      try {
        console.log('🔄 Falling back to legacy config loading...')
        const defaultConfig: WebCrawlerConfig = await invoke('get_default_config')
        setConfig(defaultConfig)
        console.log('✅ Legacy configuration loaded successfully')
      } catch (legacyError) {
        console.error('❌ Legacy config loading also failed:', legacyError)
        
        // Fallback to a default configuration for development mode
        const fallbackConfig: WebCrawlerConfig = {
        accepted_languages: ["en"],
        target_words: ["technology", "innovation"],
        min_word_length: 3,
        min_content_length: 100,
        keyword_match_all: false,
        language_content_percentage: 70,
        user_agent: "Mozilla/5.0 (compatible; WebCrawler/1.0)",
        max_crawl_depth: 3,
        max_total_urls: 100,
        enable_extension_crawling: false,
        enable_keyword_filtering: false,
        avoid_url_extensions: [".pdf", ".doc", ".zip"],
        latin_word_filter: {
          exclude_numeric: false,
          excluded_words: ["the", "and", "or", "but"],
          min_word_length: 3
        },
        // Advanced configuration defaults
        proxy_pool: [],
        max_requests_per_second: 5,
        window_size_ms: 1000,
        max_retries: 3,
        base_delay_ms: 1000,
        max_delay_ms: 30000,
        backoff_multiplier: 1.5,
        jitter_factor: 0.1,
        max_file_size_mb: 100,
        data_output_extension: 'json',
        enable_metrics: false,
        metrics_port: 9090,
        health_check_interval_secs: 30,
        persist_queue: false,
        checkpoint_interval_secs: 60
      }
      setConfig(fallbackConfig)
      }
    }
  }

  const handleStartCrawl = async (startUrl: string, crawlerConfig: WebCrawlerConfig) => {
    setIsLoading(true)
    
    try {
      // Sanitize and validate the start URL
      const sanitizedUrl = sanitizeUrl(startUrl)
      if (!isValidUrl(sanitizedUrl)) {
        throw new Error('Invalid URL format')
      }

      // Validate and sanitize the configuration
      const sanitizedConfig = sanitizeConfig(crawlerConfig)
      
      try {
        await invoke('start_crawl', { 
          startUrl: sanitizedUrl, 
          config: sanitizedConfig 
        })
        
        // Start polling for status
        const status: CrawlStatus = await invoke('get_crawl_status')
        setCrawlStatus(status)
      } catch (error) {
        // Fallback for development mode
        console.warn('Tauri backend not available, simulating crawl start:', error)
        const mockStatus: CrawlStatus = {
          total_pages: 0,
          processed_pages: 0,
          queue_size: 1,
          is_running: true
        }
        setCrawlStatus(mockStatus)
        
        // Simulate some progress after a delay
        setTimeout(() => {
          const mockProgress: CrawlStatus = {
            total_pages: 5,
            processed_pages: 2,
            queue_size: 3,
            is_running: true
          }
          setCrawlStatus(mockProgress)
        }, 2000)
      }
    } catch (error) {
      console.error('Failed to start crawl:', error)
      alert('Failed to start crawl: ' + error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleStopCrawl = async () => {
    try {
      await invoke('stop_crawl')
      const status: CrawlStatus = await invoke('get_crawl_status')
      setCrawlStatus(status)
    } catch (error) {
      console.warn('Tauri backend not available, simulating crawl stop:', error)
      const mockStatus: CrawlStatus = {
        total_pages: 5,
        processed_pages: 5,
        queue_size: 0,
        is_running: false
      }
      setCrawlStatus(mockStatus)
    }
  }

  const handleConfigChange = (newConfig: WebCrawlerConfig) => {
    setConfig(sanitizeConfig(newConfig))
  }

  const handleApplyPreset = async (presetName: string) => {
    try {
      const presetConfig: WebCrawlerConfig = await invoke('get_preset_config', { 
        preset: presetName 
      })
      setConfig(sanitizeConfig(presetConfig))
    } catch (error) {
      console.warn('Tauri backend not available, using mock preset:', error)
      
      // Helper function to create base config with all required fields
      const createBaseConfig = (overrides: Partial<WebCrawlerConfig> = {}): WebCrawlerConfig => ({
        accepted_languages: ["en"],
        target_words: ["technology", "innovation"],
        min_word_length: 3,
        min_content_length: 100,
        keyword_match_all: false,
        language_content_percentage: 70,
        user_agent: "Mozilla/5.0 (compatible; WebCrawler/1.0)",
        max_crawl_depth: 3,
        max_total_urls: 100,
        enable_extension_crawling: false,
        enable_keyword_filtering: false,
        avoid_url_extensions: [".pdf", ".doc", ".zip"],
        latin_word_filter: {
          exclude_numeric: false,
          excluded_words: ["the", "and", "or", "but"],
          min_word_length: 3
        },
        proxy_pool: [],
        max_requests_per_second: 5,
        window_size_ms: 1000,
        max_retries: 3,
        base_delay_ms: 1000,
        max_delay_ms: 30000,
        backoff_multiplier: 1.5,
        jitter_factor: 0.1,
        max_file_size_mb: 100,
        data_output_extension: 'json',
        enable_metrics: false,
        metrics_port: 9090,
        health_check_interval_secs: 30,
        persist_queue: false,
        checkpoint_interval_secs: 60,
        ...overrides
      })
      
      // Mock preset configurations for development
      const mockPresets: Record<string, WebCrawlerConfig> = {
        production: createBaseConfig({
          accepted_languages: ["en", "fr", "de"],
          target_words: ["technology", "innovation", "software", "development"],
          min_word_length: 4,
          user_agent: "Mozilla/5.0 (compatible; ProductionCrawler/2.0)",
          max_crawl_depth: 5,
          max_total_urls: 1000,
          enable_extension_crawling: false,
          enable_keyword_filtering: false,
          avoid_url_extensions: [".pdf", ".doc", ".zip", ".exe"],
          latin_word_filter: {
            exclude_numeric: true,
            excluded_words: ["the", "and", "or", "but", "in", "on", "at"],
            min_word_length: 4
          },
          proxy_pool: ["http://proxy1.example.com:8080", "http://proxy2.example.com:8080"],
          max_requests_per_second: 10,
          max_retries: 5,
          base_delay_ms: 2000,
          max_delay_ms: 60000,
          max_file_size_mb: 200,
          data_output_extension: 'csv',
          enable_metrics: true,
          persist_queue: true,
          checkpoint_interval_secs: 30
        }),
        development: createBaseConfig({
          accepted_languages: ["en"],
          target_words: ["test", "demo"],
          min_word_length: 3,
          user_agent: "Mozilla/5.0 (compatible; DevCrawler/1.0)",
          max_crawl_depth: 2,
          max_total_urls: 50,
          enable_extension_crawling: false,
          enable_keyword_filtering: false,
          avoid_url_extensions: [".pdf", ".doc"],
          latin_word_filter: {
            exclude_numeric: false,
            excluded_words: ["the", "and"],
            min_word_length: 3
          }
        }),
        demo: createBaseConfig({
          accepted_languages: ["en"],
          target_words: ["example"],
          min_word_length: 2,
          user_agent: "Mozilla/5.0 (compatible; DemoCrawler/1.0)",
          max_crawl_depth: 1,
          max_total_urls: 10,
          enable_extension_crawling: false,
          enable_keyword_filtering: false,
          avoid_url_extensions: [".pdf"],
          latin_word_filter: {
            exclude_numeric: false,
            excluded_words: [],
            min_word_length: 2
          }
        })
      }
      
      const mockConfig = mockPresets[presetName] || mockPresets.demo
      setConfig(sanitizeConfig(mockConfig))
    }
  }

  // Input sanitization functions
  const sanitizeUrl = (url: string): string => {
    const trimmed = url.trim()
    // Remove potentially dangerous characters
    const sanitized = trimmed.replace(/[<>'"]/g, '')
    // Ensure protocol is present
    if (!sanitized.startsWith('http://') && !sanitized.startsWith('https://')) {
      return `https://${sanitized}`
    }
    return sanitized
  }

  const isValidUrl = (url: string): boolean => {
    try {
      const urlObj = new URL(url)
      return ['http:', 'https:'].includes(urlObj.protocol)
    } catch {
      return false
    }
  }

  const sanitizeConfig = (config: WebCrawlerConfig): WebCrawlerConfig => {
    return {
      ...config,
      target_words: config.target_words.map(word => sanitizeString(word)),
      user_agent: sanitizeString(config.user_agent),
      avoid_url_extensions: config.avoid_url_extensions.map(ext => sanitizeString(ext)),
      accepted_languages: config.accepted_languages.map(lang => sanitizeLanguageCode(lang)),
      min_word_length: Math.max(1, Math.min(1000, config.min_word_length)),
      max_crawl_depth: Math.max(1, Math.min(10, config.max_crawl_depth)),
      max_total_urls: Math.max(1, Math.min(10000, config.max_total_urls)),
      latin_word_filter: {
        ...config.latin_word_filter,
        excluded_words: config.latin_word_filter.excluded_words.map(word => sanitizeString(word)),
        min_word_length: Math.max(1, Math.min(50, config.latin_word_filter.min_word_length))
      }
    }
  }

  const sanitizeString = (input: string): string => {
    return input
      .trim()
      .replace(/[<>'"&]/g, '') // Remove potentially dangerous characters
      .substring(0, 500) // Limit length
  }

  const sanitizeLanguageCode = (lang: string): string => {
    const validLanguages = ['en', 'fr', 'de', 'zh', 'ja', 'ko', 'es', 'it', 'pt', 'ru', 'ar', 'hi']
    const sanitized = sanitizeString(lang).toLowerCase()
    return validLanguages.includes(sanitized) ? sanitized : 'en'
  }

  if (!config) {
    return (
      <div className="min-vh-100 bg-light d-flex align-items-center justify-content-center">
        <div className="text-center">
          <div className="spinner-border text-primary" role="status" style={{ width: '3rem', height: '3rem' }}>
            <span className="visually-hidden">Loading...</span>
          </div>
          <p className="mt-3 text-muted">Loading configuration...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-vh-100 bg-light">
      {/* Google-style Header */}
      <nav className="navbar navbar-expand-lg navbar-light bg-white shadow-sm border-bottom">
        <div className="container-fluid" style={{ maxWidth: '75%' }}>
          <span className="navbar-brand mb-0 h1 google-header fw-normal">
            🕷️ Rust Web Crawler
          </span>
          <button
            className="btn btn-google-secondary d-lg-none"
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
          >
            <i className="bi bi-gear"></i> Settings
          </button>
        </div>
      </nav>

      <div className="container-fluid" style={{ maxWidth: '75%' }}>
        <div className="row g-4 py-4">
          {/* Main Content Area */}
          <div className="col-lg-8 col-12">
            <div className="fade-in">
              <CrawlerDashboard
                config={config}
                crawlStatus={crawlStatus}
                isLoading={isLoading}
                onStartCrawl={handleStartCrawl}
                onStopCrawl={handleStopCrawl}
                onConfigChange={handleConfigChange}
              />
            </div>
          </div>

          {/* Settings Sidebar */}
          <div className={`col-lg-4 col-12 ${isSidebarOpen ? 'd-block' : 'd-none d-lg-block'}`}>
            <div className="fade-in" style={{ animationDelay: '0.1s' }}>
              <SettingsSidebar
                isOpen={true}
                onClose={() => setIsSidebarOpen(false)}
                config={config}
                onConfigChange={handleConfigChange}
                onApplyPreset={handleApplyPreset}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Mobile Settings Overlay */}
      {isSidebarOpen && (
        <div 
          className="d-lg-none position-fixed top-0 start-0 w-100 h-100 bg-dark bg-opacity-50"
          style={{ zIndex: 1050 }}
          onClick={() => setIsSidebarOpen(false)}
        >
          <div 
            className="position-absolute top-0 end-0 h-100 bg-white shadow-lg"
            style={{ width: '80%', maxWidth: '400px' }}
            onClick={(e) => e.stopPropagation()}
          >
            <SettingsSidebar
              isOpen={true}
              onClose={() => setIsSidebarOpen(false)}
              config={config}
              onConfigChange={handleConfigChange}
              onApplyPreset={handleApplyPreset}
            />
          </div>
        </div>
      )}
    </div>
  )
}

export default App
