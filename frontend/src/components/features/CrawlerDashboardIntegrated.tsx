import { useState, useEffect } from 'react'
import { Toggle, MaterialInput } from '../ui'
import { CrawlerAPI, debugTauriEnvironment, testTauriCommand } from '../../api/crawler'
import { CrawlerFormConfig, CrawlStatus } from '../../types/crawler'
import { configClient } from '../../api/configuration'

interface CrawlerDashboardProps {
  onConfigChange?: (config: CrawlerFormConfig) => void
}

function CrawlerDashboard({ onConfigChange }: CrawlerDashboardProps) {
  // Form state that will be initialized from backend configuration
  const [formConfig, setFormConfig] = useState<CrawlerFormConfig>({
    baseUrl: '',
    maxTotalUrls: 100,
    maxCrawlDepth: 3,
    enableDiscoveryCrawling: true,
    enableKeywordFiltering: false,
    targetWords: [],
    enableContentFiltering: false,
    avoidUrlExtensions: ['pdf', 'doc', 'zip'],
    enableLanguageFiltering: false,
    latinWordFilter: false,
    matchStrategy: 'any',
    // These will be overridden by backend configuration
    userAgent: "",
    proxyPool: [],
    acceptedLanguages: [],
    minWordLength: 3, // Will be overridden by backend config
    defaultRateLimit: {
      maxRequestsPerSecond: 2,
      windowSizeMs: 1000,
    },
    retryConfig: {
      maxRetries: 3,
      baseDelayMs: 1000,
      maxDelayMs: 30000,
      backoffMultiplier: 2,
      jitterFactor: 0.1,
    },
    loggingConfig: {
      level: 'info',
      filePath: 'logs/crawler.log',
      jsonFormat: false,
    },
  })

  // Crawl session state
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null)
  const [crawlStatus, setCrawlStatus] = useState<CrawlStatus | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [validationError, setValidationError] = useState<string | null>(null)

  // Check Tauri environment on component mount
  useEffect(() => {
    console.log('🔍 Component mounted, checking Tauri environment...');
    const debugInfo = debugTauriEnvironment();
    console.log('Debug info:', debugInfo);
    
    console.log('Window object:', typeof window !== 'undefined' ? {
      protocol: window.location?.protocol,
      userAgent: window.navigator?.userAgent,
      __TAURI__: (window as any).__TAURI__,
      __TAURI_INVOKE__: (window as any).__TAURI_INVOKE__,
    } : 'undefined');
  }, []);

  // Load configuration from backend
  useEffect(() => {
    const loadConfiguration = async () => {
      try {
        console.log('🔧 Loading configuration from backend...');
        await configClient.loadConfiguration();
        const backendDefaults = configClient.getDefaultFormValues();
        
        console.log('✅ Configuration loaded, updating form defaults:', backendDefaults);
        
        setFormConfig(prev => ({
          ...prev,
          userAgent: backendDefaults.userAgent,
          maxTotalUrls: backendDefaults.maxUrls,
          maxCrawlDepth: backendDefaults.maxDepth,
          minWordLength: backendDefaults.minWordLength, // This will now be 3 instead of 10
          acceptedLanguages: backendDefaults.acceptedLanguages,
          proxyPool: backendDefaults.proxyPool,
        }));
      } catch (error) {
        console.error('❌ Failed to load configuration:', error);
        // Keep the default values if configuration loading fails
      }
    };

    loadConfiguration();
  }, []);

  // Poll for status updates
  useEffect(() => {
    let pollInterval: number | null = null

    if (currentSessionId && crawlStatus?.status === 'running') {
      pollInterval = setInterval(async () => {
        try {
          const status = await CrawlerAPI.getCrawlStatus(currentSessionId)
          setCrawlStatus(status)
          
          // Stop polling if crawl is complete
          if (status.status !== 'running') {
            setCurrentSessionId(null)
            if (pollInterval) clearInterval(pollInterval)
          }
        } catch (error) {
          console.error('Error polling status:', error)
        }
      }, 2000)
    }

    return () => {
      if (pollInterval) clearInterval(pollInterval)
    }
  }, [currentSessionId, crawlStatus?.status])

  // Form submission handler
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    console.log('🚀 Form submission started with config:', formConfig)
    console.log('📊 Form config field details:')
    console.log('  - baseUrl:', formConfig.baseUrl)
    console.log('  - userAgent:', formConfig.userAgent)
    console.log('  - proxyPool:', formConfig.proxyPool)
    console.log('  - acceptedLanguages:', formConfig.acceptedLanguages)
    console.log('  - minWordLength:', formConfig.minWordLength)
    console.log('  - defaultRateLimit:', formConfig.defaultRateLimit)
    console.log('  - retryConfig:', formConfig.retryConfig)
    console.log('  - loggingConfig:', formConfig.loggingConfig)
    console.log('🏁 Form submission completed, isLoading set to false')
    setValidationError(null)
    setIsLoading(true)

    try {
      console.log('📡 Calling CrawlerAPI.startAndExecuteCrawl...')
      // Start and execute the crawl
      const result = await CrawlerAPI.startAndExecuteCrawl(formConfig)
      console.log('✅ API call successful, result:', result)
      
      setCurrentSessionId(result.sessionId)
      setCrawlStatus({
        session_id: result.sessionId,
        status: 'running',
        total_urls_processed: 0,
        successful_crawls: 0,
        failed_crawls: 0,
        errors: [],
        results: [],
      })

      console.log('🎯 Crawl started successfully with session ID:', result.sessionId)
      
    } catch (error) {
      console.error('❌ Crawl failed with error:', error)
      setValidationError(error instanceof Error ? error.message : 'Failed to start crawl')
    } finally {
      console.log('🏁 Form submission completed, isLoading set to false')
      setIsLoading(false)
    }
  }

  // Stop crawl handler
  const handleStopCrawl = async () => {
    if (!currentSessionId) return

    try {
      await CrawlerAPI.stopCrawl(currentSessionId)
      setCrawlStatus(prev => prev ? { ...prev, status: 'idle' } : null)
      setCurrentSessionId(null)
    } catch (error) {
      console.error('Failed to stop crawl:', error)
    }
  }

  // Test Tauri connection directly
  const handleTestTauri = async () => {
    console.log('🧪 Testing Tauri connection...');
    try {
      // Try to call a simple command directly
      const result = await testTauriCommand('get_default_config');
      console.log('✅ Tauri test successful:', result);
      alert('Tauri connection test successful!');
    } catch (error) {
      console.error('❌ Tauri test failed:', error);
      alert(`Tauri test failed: ${error}`);
    }
  }

  // Input validation and sanitization functions
  const sanitizeString = (input: string): string => {
    if (!input || typeof input !== 'string') return ''
    
    return input
      .replace(/[<>"'%;()&+]/g, '') // Remove dangerous HTML/SQL chars
      .replace(/--/g, '')
      .replace(/\/\*/g, '') // Remove SQL comment start
      .replace(/\*\//g, '') // Remove SQL comment end
      .trim()
      .slice(0, 1000)
  }

  const sanitizeUrl = (url: string): string => {
    if (!url || typeof url !== 'string') return ''
    
    const cleanUrl = url.trim().slice(0, 2048)
    
    if (!cleanUrl.match(/^https?:\/\//i)) {
      return cleanUrl.startsWith('//') ? `https:${cleanUrl}` : `https://${cleanUrl}`
    }
    
    return cleanUrl
  }

  const validateNumericInput = (value: string, min: number = 0, max: number = Number.MAX_SAFE_INTEGER): number => {
    const num = parseInt(value) || min
    return Math.max(min, Math.min(max, num))
  }

  const validateAndSanitizeKeywords = (keywords: string): string[] => {
    if (!keywords || typeof keywords !== 'string') return []
    
    return keywords
      .split(',')
      .map(keyword => sanitizeString(keyword))
      .filter(keyword => keyword.length > 0 && keyword.length <= 100)
      .slice(0, 50)
  }

  // Form field change handler
  const handleFieldChange = (field: keyof CrawlerFormConfig, value: any) => {
    const newConfig = { ...formConfig }
    
    try {
      if (field === 'targetWords') {
        newConfig.targetWords = validateAndSanitizeKeywords(value)
      } else if (field === 'avoidUrlExtensions') {
        newConfig.avoidUrlExtensions = validateAndSanitizeKeywords(value)
      } else if (field === 'baseUrl') {
        newConfig.baseUrl = sanitizeUrl(value)
      } else if (field === 'maxTotalUrls' || field === 'maxCrawlDepth') {
        newConfig[field] = validateNumericInput(value.toString(), 1, field === 'maxTotalUrls' ? 10000 : 10)
      } else if (typeof value === 'boolean') {
        // Type assertion for boolean fields
        (newConfig as any)[field] = value
      } else if (typeof value === 'string') {
        (newConfig as any)[field] = sanitizeString(value)
      } else {
        (newConfig as any)[field] = value
      }
      
      setFormConfig(newConfig)
      onConfigChange?.(newConfig)
    } catch (error) {
      console.error('Error updating configuration:', error)
    }
  }

  console.log('🔍 Current crawlStatus:', crawlStatus);

  return (
    <div className="d-flex flex-column gap-4">
      {/* Debug Section */}
      <div className="card-google">
        <div className="d-flex gap-2 align-items-center">
          <button
            type="button"
            onClick={handleTestTauri}
            className="btn btn-outline-info btn-sm"
          >
            🧪 Test Tauri Connection
          </button>
          <small className="text-muted">
            Use this to test if Tauri APIs are working before submitting the form
          </small>
        </div>
      </div>

      {/* Main Crawl Form */}
      <div className="card-google">
        <form onSubmit={handleSubmit}>
          {/* Primary URL Input */}
          <div className="mb-5 mt-4">
            <div className="primary-url-container">
              <MaterialInput
                id="baseUrl"
                label="Enter URL to crawl"
                value={formConfig.baseUrl}
                onChange={(value) => handleFieldChange('baseUrl', value)}
                type="url"
                required
                className="primary-url-input"
              />
              {formConfig.baseUrl && (() => {
                // Handle comma-separated URLs - validate first URL only for display
                const firstUrl = formConfig.baseUrl.split(',')[0].trim()
                try {
                  new URL(firstUrl)
                  return null
                } catch {
                  return (
                    <div className="form-text mt-2">
                      <small className="text-warning">
                        <i className="bi bi-exclamation-triangle me-1"></i>
                        Please enter a valid URL (e.g., https://example.com)
                      </small>
                    </div>
                  )
                }
              })()}
            </div>
          </div>

          {/* Basic Configuration */}
          <div className="row g-4 mb-4">
            <div className="col-md-6">
              <div className="material-group">
                <input
                  id="maxTotalUrls"
                  type="number"
                  className="material-input"
                  value={formConfig.maxTotalUrls}
                  onChange={(e) => handleFieldChange('maxTotalUrls', parseInt(e.target.value) || 1)}
                  min="1"
                  max="10000"
                  placeholder=" "
                />
                <label htmlFor="maxTotalUrls" className="material-label">Maximum Total URLs</label>
                <span className="material-bar"></span>
              </div>
            </div>

            <div className="col-md-6">
              <div className="material-group">
                <input
                  id="maxCrawlDepth"
                  type="number"
                  className="material-input"
                  value={formConfig.maxCrawlDepth}
                  onChange={(e) => handleFieldChange('maxCrawlDepth', parseInt(e.target.value) || 1)}
                  min="1"
                  max="10"
                  placeholder=" "
                />
                <label htmlFor="maxCrawlDepth" className="material-label">Maximum Crawl Depth</label>
                <span className="material-bar"></span>
              </div>
            </div>
          </div>

          {/* Feature Toggles */}
          <div className="d-flex flex-column gap-4 mb-4">
            <Toggle
              enabled={formConfig.enableDiscoveryCrawling}
              onChange={(enabled) => handleFieldChange('enableDiscoveryCrawling', enabled)}
              label="Discovery Crawling"
              description="Automatically discover and queue new URLs during crawling"
            />

            <Toggle
              enabled={formConfig.enableKeywordFiltering}
              onChange={(enabled) => handleFieldChange('enableKeywordFiltering', enabled)}
              label="Target Keyword Filter"
              description="Only crawl pages containing target keywords"
            />

            <Toggle
              enabled={formConfig.enableContentFiltering}
              onChange={(enabled) => handleFieldChange('enableContentFiltering', enabled)}
              label="Content Filter"
              description="Filter out specific file extensions and apply content rules"
            />

            <Toggle
              enabled={formConfig.enableLanguageFiltering}
              onChange={(enabled) => handleFieldChange('enableLanguageFiltering', enabled)}
              label="Language-Focus Filter"
              description="Apply language-specific filtering"
            />
          </div>

          {/* Conditional Sections */}
          
          {/* Keyword Filtering Section */}
          <div className={`filter-section ${formConfig.enableKeywordFiltering ? 'filter-section-visible' : ''}`}>
            <div className="filter-section-content">
              <h5 className="google-subheader mb-4">Keyword Filtering Options</h5>
              <div className="row g-4">
                <div className="col-12">
                  <div className="material-group">
                    <input
                      id="targetWords"
                      type="text"
                      className={`material-input ${formConfig.enableKeywordFiltering && formConfig.targetWords.length === 0 ? 'is-invalid' : ''}`}
                      value={formConfig.targetWords.join(', ')}
                      onChange={(e) => handleFieldChange('targetWords', e.target.value)}
                      placeholder=" "
                      maxLength={1000}
                    />
                    <label htmlFor="targetWords" className="material-label">Target Keywords (comma-separated)</label>
                    <span className="material-bar"></span>
                    {formConfig.enableKeywordFiltering && formConfig.targetWords.length === 0 && (
                      <div className="invalid-feedback d-block">
                        <small className="text-danger">Keywords are required when keyword filtering is enabled</small>
                      </div>
                    )}
                  </div>
                </div>

                <div className="col-12">
                  <div className="material-group">
                    <select
                      value={formConfig.matchStrategy}
                      onChange={(e) => handleFieldChange('matchStrategy', e.target.value as 'any' | 'all')}
                      className="material-input"
                    >
                      <option value="any">Any Keyword</option>
                      <option value="all">All Keywords</option>
                    </select>
                    <label className="material-label">Match must contain</label>
                    <span className="material-bar"></span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Content Filtering Section */}
          <div className={`filter-section ${formConfig.enableContentFiltering ? 'filter-section-visible' : ''}`}>
            <div className="filter-section-content">
              <h5 className="google-subheader mb-4">Content Filtering Options</h5>
              <div className="row g-4">
                <div className="col-12">
                  <div className="material-group">
                    <input
                      id="avoidExtensions"
                      type="text"
                      className="material-input"
                      value={formConfig.avoidUrlExtensions.join(', ')}
                      onChange={(e) => handleFieldChange('avoidUrlExtensions', e.target.value)}
                      placeholder=" "
                    />
                    <label htmlFor="avoidExtensions" className="material-label">Avoid File Extensions (comma-separated)</label>
                    <span className="material-bar"></span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Language Filtering Section */}
          <div className={`filter-section ${formConfig.enableLanguageFiltering ? 'filter-section-visible' : ''}`}>
            <div className="filter-section-content">
              <h5 className="google-subheader mb-4">Language Filtering Options</h5>
              <div className="row g-4">
                <div className="col-12">
                  <Toggle
                    enabled={formConfig.latinWordFilter}
                    onChange={(enabled) => handleFieldChange('latinWordFilter', enabled)}
                    label="Latin Word Filter"
                    description="Apply Latin-based language filtering rules"
                  />
                </div>
              </div>
            </div>
          </div>

          {/* Validation Error Display */}
          {validationError && (
            <div className="alert alert-danger mb-4">
              <i className="bi bi-exclamation-triangle me-2"></i>
              {validationError}
            </div>
          )}

          {/* Start Crawl Button */}
          <div className="pt-4 mt-4 border-top">
            <button
              type="submit"
              disabled={isLoading || crawlStatus?.status === 'running' || !formConfig.baseUrl.trim()}
              className="btn btn-google-primary w-100 py-3"
            >
              {isLoading ? (
                <>
                  <span className="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                  Starting Crawl...
                </>
              ) : (
                <>
                  <i className="bi bi-play-fill me-2"></i>
                  Start Web Crawl
                </>
              )}
            </button>
          </div>
        </form>
      </div>

      {/* Status Display */}
      {crawlStatus && (
        <div className="card-google">
          <div className="d-flex align-items-center justify-content-between mb-4">
            <h2 className="google-header h5 mb-0">Crawl Status</h2>
            {crawlStatus.status === 'running' && (
              <button
                onClick={handleStopCrawl}
                className="btn btn-outline-danger btn-sm d-flex align-items-center"
              >
                <i className="bi bi-stop-circle me-1"></i>
                Stop Crawl
              </button>
            )}
          </div>

          <div className="row g-3">
            <div className="col-6 col-md-3">
              <div className="bg-primary bg-opacity-10 p-3 rounded text-center">
                <div className="h4 text-primary mb-1">
                  {crawlStatus.total_urls_processed}
                </div>
                <div className="small text-muted">Processed</div>
              </div>
            </div>
            
            <div className="col-6 col-md-3">
              <div className="bg-success bg-opacity-10 p-3 rounded text-center">
                <div className="h4 text-success mb-1">
                  {crawlStatus.successful_crawls}
                </div>
                <div className="small text-muted">Successful</div>
              </div>
            </div>
            
            <div className="col-6 col-md-3">
              <div className="bg-danger bg-opacity-10 p-3 rounded text-center">
                <div className="h4 text-danger mb-1">
                  {crawlStatus.failed_crawls}
                </div>
                <div className="small text-muted">Failed</div>
              </div>
            </div>
            
            <div className="col-6 col-md-3">
              <div className={`p-3 rounded text-center ${
                crawlStatus.status === 'running' 
                  ? 'bg-success bg-opacity-10' 
                  : 'bg-secondary bg-opacity-10'
              }`}>
                <div className={`h4 mb-1 ${
                  crawlStatus.status === 'running' 
                    ? 'text-success' 
                    : 'text-secondary'
                }`}>
                  {crawlStatus.status === 'running' ? (
                    <div className="d-flex align-items-center justify-content-center">
                      <span className="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                      {crawlStatus.status.toUpperCase()}
                    </div>
                  ) : (
                    crawlStatus.status.toUpperCase()
                  )}
                </div>
                <div className="small text-muted">Status</div>
              </div>
            </div>
          </div>

          {/* Current URL */}
          {crawlStatus.current_url && (
            <div className="mt-3">
              <div className="d-flex align-items-center">
                <i className="bi bi-globe me-2 text-muted"></i>
                <small className="text-muted">Currently crawling:</small>
              </div>
              <div className="mt-1">
                <code className="small bg-light p-2 rounded d-block">
                  {crawlStatus.current_url}
                </code>
              </div>
            </div>
          )}

          {/* Errors */}
          {crawlStatus.errors.length > 0 && (
            <div className="mt-3">
              <div className="d-flex align-items-center mb-2">
                <i className="bi bi-exclamation-triangle me-2 text-warning"></i>
                <small className="text-muted">Errors ({crawlStatus.errors.length}):</small>
              </div>
              <div className="bg-light p-3 rounded" style={{ maxHeight: '200px', overflowY: 'auto' }}>
                {crawlStatus.errors.map((error, index) => (
                  <div key={index} className="small text-danger mb-1">
                    {error}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Results Summary */}
          {crawlStatus.results.length > 0 && (
            <div className="mt-4">
              <div className="d-flex align-items-center mb-3">
                <i className="bi bi-list-check me-2 text-success"></i>
                <span className="fw-medium">Results ({crawlStatus.results.length})</span>
              </div>
              <div className="bg-light p-3 rounded" style={{ maxHeight: '300px', overflowY: 'auto' }}>
                {crawlStatus.results.map((result, index) => (
                  <div key={index} className="mb-3 pb-3 border-bottom border-light">
                    <div className="d-flex justify-content-between align-items-start mb-2">
                      <div className="fw-medium small">{result.title || 'Untitled'}</div>
                      <span className="badge bg-secondary">{result.status_code}</span>
                    </div>
                    <div className="small text-muted mb-1">{result.url}</div>
                    <div className="small">
                      Words: {result.word_count} | 
                      Keywords found: {result.target_words_found.length}
                      {result.language && ` | Language: ${result.language}`}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default CrawlerDashboard
