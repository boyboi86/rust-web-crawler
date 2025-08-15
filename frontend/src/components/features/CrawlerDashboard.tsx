import { useState } from 'react'
import { WebCrawlerConfig, CrawlStatus } from '../../App'
import { Toggle, MaterialInput } from '../ui'

interface CrawlerDashboardProps {
  config: WebCrawlerConfig
  crawlStatus: CrawlStatus | null
  isLoading: boolean
  onStartCrawl: (startUrl: string, config: WebCrawlerConfig) => void
  onStopCrawl: () => void
  onConfigChange: (config: WebCrawlerConfig) => void
}

function CrawlerDashboard({
  config,
  crawlStatus,
  isLoading,
  onStartCrawl,
  onStopCrawl,
  onConfigChange
}: CrawlerDashboardProps) {
  const [startUrl, setStartUrl] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!startUrl.trim()) {
      alert('Please enter a valid URL to crawl')
      return
    }

    // Validate and sanitize URLs
    const urls = startUrl.split(',')
      .map(url => sanitizeUrl(url))
      .filter(url => {
        // Basic URL validation
        try {
          new URL(url)
          return true
        } catch {
          return false
        }
      })

    if (urls.length === 0) {
      alert('Please enter at least one valid URL (e.g., https://example.com)')
      return
    }

    // Additional validation for required fields when filters are enabled
    if (config.enable_keyword_filtering && config.target_words.length === 0) {
      alert('Please enter target keywords when keyword filtering is enabled')
      return
    }

    const sanitizedUrl = urls.join(',')
    onStartCrawl(sanitizedUrl, config)
  }

  // Input validation and sanitization functions
  const sanitizeString = (input: string): string => {
    if (!input || typeof input !== 'string') return '';
    
    // Remove potentially dangerous characters for SQL injection and XSS
    return input
      .replace(/[<>\"'%;()&+]/g, '') // Remove dangerous HTML/SQL chars
      .replace(/--/g, '') // Remove SQL comment markers
      .replace(/\/\*/g, '') // Remove SQL comment start
      .replace(/\*\//g, '') // Remove SQL comment end
      .trim()
      .slice(0, 1000); // Limit length
  };

  const sanitizeUrl = (url: string): string => {
    if (!url || typeof url !== 'string') return '';
    
    // Basic URL sanitization
    const cleanUrl = url.trim().slice(0, 2048); // Limit URL length
    
    // Check if URL has valid protocol
    if (!cleanUrl.match(/^https?:\/\//i)) {
      return cleanUrl.startsWith('//') ? `https:${cleanUrl}` : `https://${cleanUrl}`;
    }
    
    return cleanUrl;
  };

  const validateNumericInput = (value: string, min: number = 0, max: number = Number.MAX_SAFE_INTEGER): number => {
    const num = parseInt(value) || min;
    return Math.max(min, Math.min(max, num));
  };

  const validateAndSanitizeKeywords = (keywords: string): string[] => {
    if (!keywords || typeof keywords !== 'string') return [];
    
    return keywords
      .split(',')
      .map(keyword => sanitizeString(keyword))
      .filter(keyword => keyword.length > 0 && keyword.length <= 100) // Reasonable keyword length
      .slice(0, 50); // Limit number of keywords
  };

  const handleConfigFieldChange = (field: string, value: any) => {
    const newConfig = { ...config }
    
    try {
      if (field === 'target_words') {
        newConfig.target_words = validateAndSanitizeKeywords(value)
      } else if (field === 'avoid_url_extensions') {
        newConfig.avoid_url_extensions = validateAndSanitizeKeywords(value)
      } else if (field === 'latin_word_filter.exclude_numeric') {
        newConfig.latin_word_filter = {
          ...newConfig.latin_word_filter,
          exclude_numeric: Boolean(value)
        }
      } else if (field === 'latin_word_filter.excluded_words') {
        newConfig.latin_word_filter = {
          ...newConfig.latin_word_filter,
          excluded_words: validateAndSanitizeKeywords(value)
        }
      } else if (field === 'min_word_length') {
        newConfig.min_word_length = validateNumericInput(value.toString(), 1, 100)
      } else if (field === 'min_content_length') {
        newConfig.min_content_length = validateNumericInput(value.toString(), 1, 50000)
      } else if (field === 'language_content_percentage') {
        newConfig.language_content_percentage = validateNumericInput(value.toString(), 1, 100)
      } else if (field === 'accepted_languages') {
        // Validate language codes
        const validLanguages = ['en', 'es', 'fr', 'de', 'it', 'pt', 'ru', 'ja', 'ko', 'zh', 'ar', 'hi'];
        if (Array.isArray(value)) {
          newConfig.accepted_languages = value.filter(lang => validLanguages.includes(lang));
        } else if (typeof value === 'string' && validLanguages.includes(value)) {
          newConfig.accepted_languages = [value];
        } else {
          newConfig.accepted_languages = [];
        }
      } else if (field === 'keyword_match_all') {
        newConfig.keyword_match_all = Boolean(value)
      } else if (field === 'enable_keyword_filtering' || field === 'enable_extension_crawling') {
        (newConfig as any)[field] = Boolean(value)
      } else {
        // For other fields, apply basic sanitization
        if (typeof value === 'string') {
          (newConfig as any)[field] = sanitizeString(value)
        } else {
          (newConfig as any)[field] = value
        }
      }
      
      onConfigChange(newConfig)
    } catch (error) {
      console.error('Error updating configuration:', error)
      // Don't update config if there's an error
    }
  }

  return (
    <div className="d-flex flex-column gap-4">
      {/* Main Crawl Form */}
      <div className="card-google">
        <form onSubmit={handleSubmit}>
          {/* Primary URL Input - Most Important */}
          <div className="mb-5 mt-4">
            <div className="primary-url-container">
              <MaterialInput
                id="startUrl"
                label="Enter URL to crawl (Comma-separated)"
                value={startUrl}
                onChange={(value) => setStartUrl(sanitizeString(value))}
                type="url"
                required
                className="primary-url-input"
              />
              {startUrl && !startUrl.split(',').every(url => {
                try {
                  new URL(sanitizeUrl(url.trim()))
                  return true
                } catch {
                  return false
                }
              }) && (
                <div className="form-text mt-2">
                  <small className="text-warning">
                    <i className="bi bi-exclamation-triangle me-1"></i>
                    Please enter valid URLs (e.g., https://example.com)
                  </small>
                </div>
              )}
            </div>
          </div>

          {/* Essential Behavior Toggles */}
          <div className="d-flex flex-column gap-4 mb-4">
            <Toggle
              enabled={config.enable_extension_crawling}
              onChange={(enabled) => handleConfigFieldChange('enable_extension_crawling', enabled)}
              label="Discovery Crawling"
              description="Automatically discover and queue new URLs during crawling"
            />

            <Toggle
              enabled={config.enable_keyword_filtering}
              onChange={(enabled) => handleConfigFieldChange('enable_keyword_filtering', enabled)}
              label="Target Keyword Filter"
              description="Only crawl pages containing target keywords"
            />



            <Toggle
              enabled={config.latin_word_filter.exclude_numeric}
              onChange={(enabled) => handleConfigFieldChange('latin_word_filter.exclude_numeric', enabled)}
              label="Content Filter"
              description="Filter out numeric words and apply content filtering rules"
            />

            <Toggle
              enabled={config.accepted_languages.length > 0}
              onChange={(enabled) => {
                if (!enabled) {
                  handleConfigFieldChange('accepted_languages', [])
                } else {
                  handleConfigFieldChange('accepted_languages', ['en'])
                }
              }}
              label="Language-Focus Filter"
              description="Apply language-specific filtering based on accepted languages"
            />
          </div>

          {/* Conditional Filter Sections */}
          
          {/* Content Filtering Section */}
          <div className={`filter-section ${config.latin_word_filter.exclude_numeric ? 'filter-section-visible' : ''}`}>
            <div className="filter-section-content">
              <h5 className="google-subheader mb-4">Content Filtering Options</h5>
              <div className="row g-4">
                <div className="col-12">
                  <div className="material-group">
                    <input
                      id="minWordLength"
                      type="number"
                      className="material-input"
                      value={config.min_word_length}
                      onChange={(e) => handleConfigFieldChange('min_word_length', parseInt(e.target.value) || 1)}
                      min="1"
                      max="50"
                      placeholder=" "
                    />
                    <label htmlFor="minWordLength" className="material-label">Minimum Word Length</label>
                    <span className="material-bar"></span>
                  </div>
                </div>

                <div className="col-12">
                  <div className="material-group">
                    <input
                      id="excludedWords"
                      type="text"
                      className="material-input"
                      value={config.latin_word_filter.excluded_words.join(', ')}
                      onChange={(e) => handleConfigFieldChange('latin_word_filter.excluded_words', e.target.value.split(',').map(w => w.trim()).filter(w => w))}
                      placeholder=" "
                    />
                    <label htmlFor="excludedWords" className="material-label">Excluded Words (comma-separated)</label>
                    <span className="material-bar"></span>
                  </div>
                </div>

                <div className="col-12">
                  <div className="material-group">
                    <input
                      id="minContentLength"
                      type="number"
                      className="material-input"
                      value={config.min_content_length || 100}
                      onChange={(e) => handleConfigFieldChange('min_content_length', parseInt(e.target.value) || 100)}
                      min="1"
                      max="10000"
                      placeholder=" "
                    />
                    <label htmlFor="minContentLength" className="material-label">Minimum Content Length (characters)</label>
                    <span className="material-bar"></span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Keyword Filtering Section */}
          <div className={`filter-section ${config.enable_keyword_filtering ? 'filter-section-visible' : ''}`}>
            <div className="filter-section-content">
              <h5 className="google-subheader mb-4">Keyword Filtering Options</h5>
              <div className="row g-4">
                <div className="col-12">
                  <div className="material-group">
                    <input
                      id="targetWords"
                      type="text"
                      className={`material-input ${config.enable_keyword_filtering && config.target_words.length === 0 ? 'is-invalid' : ''}`}
                      value={config.target_words.join(', ')}
                      onChange={(e) => handleConfigFieldChange('target_words', e.target.value)}
                      placeholder=" "
                      maxLength={1000}
                    />
                    <label htmlFor="targetWords" className="material-label">Target Keywords (comma-separated)</label>
                    <span className="material-bar"></span>
                    {config.enable_keyword_filtering && config.target_words.length === 0 && (
                      <div className="invalid-feedback d-block">
                        <small className="text-danger">Keywords are required when keyword filtering is enabled</small>
                      </div>
                    )}
                    {config.target_words.length > 0 && (
                      <div className="form-text">
                        <small className="text-muted">{config.target_words.length} keyword{config.target_words.length !== 1 ? 's' : ''} configured</small>
                      </div>
                    )}
                  </div>
                </div>

                <div className="col-12">
                  <div className="material-group">
                    <select
                      value={config.keyword_match_all ? 'all' : 'any'}
                      onChange={(e) => handleConfigFieldChange('keyword_match_all', e.target.value === 'all')}
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

          {/* Language Filtering Section */}
          <div className={`filter-section ${config.accepted_languages.length > 0 ? 'filter-section-visible' : ''}`}>
            <div className="filter-section-content">
              <h5 className="google-subheader mb-4">Language Filtering Options</h5>
              <div className="row g-4">
                <div className="col-12">
                  <div className="material-group">
                    <select
                      value={config.accepted_languages.length > 0 ? config.accepted_languages[0] : ''}
                      onChange={(e) => {
                        const value = e.target.value
                        if (value) {
                          handleConfigFieldChange('accepted_languages', [value])
                        } else {
                          handleConfigFieldChange('accepted_languages', [])
                        }
                      }}
                      className="material-input"
                    >
                      <option value="">Select Language...</option>
                      <option value="en">🇺🇸 English</option>
                      <option value="es">🇪🇸 Spanish</option>
                      <option value="fr">🇫🇷 French</option>
                      <option value="de">🇩🇪 German</option>
                      <option value="it">🇮🇹 Italian</option>
                      <option value="pt">🇵🇹 Portuguese</option>
                      <option value="ru">🇷🇺 Russian</option>
                      <option value="ja">🇯🇵 Japanese</option>
                      <option value="ko">🇰🇷 Korean</option>
                      <option value="zh">🇨🇳 Chinese</option>
                      <option value="ar">🇸🇦 Arabic</option>
                      <option value="hi">🇮🇳 Hindi</option>
                    </select>
                    <label className="material-label">Target Language</label>
                    <span className="material-bar"></span>
                  </div>
                </div>

                <div className="col-12">
                  <div className="material-group">
                    <input
                      id="languagePercentage"
                      type="number"
                      className="material-input"
                      value={config.language_content_percentage || 70}
                      onChange={(e) => handleConfigFieldChange('language_content_percentage', parseInt(e.target.value) || 70)}
                      min="1"
                      max="100"
                      step="5"
                      placeholder=" "
                    />
                    <label htmlFor="languagePercentage" className="material-label">Minimum Language Content (%)</label>
                    <span className="material-bar"></span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Start Crawl Button - At Bottom */}
          <div className="pt-4 mt-4 border-top">
            <button
              type="submit"
              disabled={isLoading || crawlStatus?.is_running || !startUrl.trim()}
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
            {crawlStatus.is_running && (
              <button
                onClick={onStopCrawl}
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
                  {crawlStatus.processed_pages}
                </div>
                <div className="small text-muted">Processed</div>
              </div>
            </div>
            
            <div className="col-6 col-md-3">
              <div className="bg-success bg-opacity-10 p-3 rounded text-center">
                <div className="h4 text-success mb-1">
                  {crawlStatus.total_pages}
                </div>
                <div className="small text-muted">Total Pages</div>
              </div>
            </div>
            
            <div className="col-6 col-md-3">
              <div className="bg-warning bg-opacity-10 p-3 rounded text-center">
                <div className="h4 text-warning mb-1">
                  {crawlStatus.queue_size}
                </div>
                <div className="small text-muted">In Queue</div>
              </div>
            </div>
            
            <div className="col-6 col-md-3">
              <div className={`p-3 rounded text-center ${
                crawlStatus.is_running 
                  ? 'bg-success bg-opacity-10' 
                  : 'bg-secondary bg-opacity-10'
              }`}>
                <div className={`h4 mb-1 ${
                  crawlStatus.is_running 
                    ? 'text-success' 
                    : 'text-secondary'
                }`}>
                  {crawlStatus.is_running ? (
                    <div className="d-flex align-items-center justify-content-center">
                      <span className="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                      ON
                    </div>
                  ) : (
                    'OFF'
                  )}
                </div>
                <div className="small text-muted">Status</div>
              </div>
            </div>
          </div>

          {/* Progress Bar */}
          {crawlStatus.total_pages > 0 && (
            <div className="mt-4">
              <div className="d-flex justify-content-between align-items-center mb-2">
                <span className="small text-muted">Progress</span>
                <span className="small text-muted">
                  {Math.round((crawlStatus.processed_pages / crawlStatus.total_pages) * 100)}%
                </span>
              </div>
              <div className="progress" style={{ height: '8px' }}>
                <div 
                  className="progress-bar bg-primary"
                  role="progressbar"
                  style={{ width: `${(crawlStatus.processed_pages / crawlStatus.total_pages) * 100}%` }}
                  aria-valuenow={crawlStatus.processed_pages}
                  aria-valuemin={0}
                  aria-valuemax={crawlStatus.total_pages}
                ></div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default CrawlerDashboard
