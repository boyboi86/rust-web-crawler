import { useState } from 'react'
import { WebCrawlerConfig, CrawlResult, CrawlStatus } from '../App'
import Toggle from './Toggle'

interface CrawlerDashboardProps {
  config: WebCrawlerConfig
  crawlStatus: CrawlStatus | null
  crawlResults: CrawlResult[]
  isLoading: boolean
  onStartCrawl: (startUrl: string, config: WebCrawlerConfig) => void
  onStopCrawl: () => void
  onGetResults: () => void
  onConfigChange: (config: WebCrawlerConfig) => void
}

function CrawlerDashboard({
  config,
  crawlStatus,
  crawlResults,
  isLoading,
  onStartCrawl,
  onStopCrawl,
  onGetResults,
  onConfigChange
}: CrawlerDashboardProps) {
  const [startUrl, setStartUrl] = useState('')
  const [showAdvanced, setShowAdvanced] = useState(false)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (startUrl.trim()) {
      onStartCrawl(startUrl.trim(), config)
    }
  }

  const sanitizeUrl = (url: string): string => {
    return url.trim().replace(/[<>'"]/g, '')
  }

  const handleUrlChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const sanitized = sanitizeUrl(e.target.value)
    setStartUrl(sanitized)
  }

  const handleConfigFieldChange = (field: string, value: any) => {
    const newConfig = { ...config }
    if (field === 'target_words') {
      newConfig.target_words = value.split(',').map((word: string) => word.trim()).filter((word: string) => word)
    } else if (field === 'avoid_url_extensions') {
      newConfig.avoid_url_extensions = value.split(',').map((ext: string) => ext.trim()).filter((ext: string) => ext)
    } else if (field === 'latin_word_filter.exclude_numeric') {
      newConfig.latin_word_filter = {
        ...newConfig.latin_word_filter,
        exclude_numeric: value
      }
    } else {
      (newConfig as any)[field] = value
    }
    onConfigChange(newConfig)
  }

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString()
  }

  const exportResults = () => {
    const dataStr = JSON.stringify(crawlResults, null, 2)
    const dataBlob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(dataBlob)
    const link = document.createElement('a')
    link.href = url
    link.download = `crawl-results-${new Date().toISOString().split('T')[0]}.json`
    link.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div className="d-flex flex-column gap-4">
      {/* Crawl Control Form */}
      <div className="card-google">
        <h2 className="google-header h4 mb-3">Start Web Crawl</h2>
        
        <form onSubmit={handleSubmit}>
          <div className="row g-3">
            <div className="col-12 col-lg-8">
              <div className="form-group-material">
                <input
                  type="url"
                  id="startUrl"
                  value={startUrl}
                  onChange={handleUrlChange}
                  placeholder=" "
                  required
                  className="form-control-material"
                />
                <span className="material-bar"></span>
                <label htmlFor="startUrl" className="form-label">
                  Starting URL
                </label>
              </div>
            </div>
            <div className="col-12 col-lg-4 d-flex align-items-end">
              <button
                type="submit"
                disabled={isLoading || crawlStatus?.is_running || !startUrl.trim()}
                className="btn btn-google-primary w-100 py-2"
              >
                {isLoading ? (
                  <>
                    <span className="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                    Starting...
                  </>
                ) : (
                  <>
                    <i className="bi bi-play-fill me-1"></i>
                    Start Crawl
                  </>
                )}
              </button>
            </div>
          </div>

          {/* Crawling Behavior Configuration */}
          <div className="border-top pt-4 mt-4">
            <button
              type="button"
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="btn btn-link p-0 text-decoration-none google-subheader d-flex align-items-center"
            >
              <i className={`bi bi-chevron-${showAdvanced ? 'down' : 'right'} me-2`}></i>
              Crawling Behavior Settings
            </button>

            {showAdvanced && (
              <div className="mt-4">
                <div className="form-group-material mb-4">
                  <input
                    type="text"
                    value={config.target_words.join(', ')}
                    onChange={(e) => handleConfigFieldChange('target_words', e.target.value)}
                    placeholder=" "
                    className="form-control-material"
                    id="targetKeywords"
                  />
                  <span className="material-bar"></span>
                  <label htmlFor="targetKeywords" className="form-label">
                    Target Keywords (comma-separated)
                  </label>
                </div>

                {/* Behavior Toggles */}
                <div className="row g-3">
                  <div className="col-md-6">
                    <Toggle
                      enabled={config.enable_keyword_filtering}
                      onChange={(enabled) => handleConfigFieldChange('enable_keyword_filtering', enabled)}
                      label="Target Keyword Filter"
                      description="Only crawl pages containing target keywords"
                    />
                  </div>

                  <div className="col-md-6">
                    <Toggle
                      enabled={config.enable_extension_crawling}
                      onChange={(enabled) => handleConfigFieldChange('enable_extension_crawling', enabled)}
                      label="Extension Crawling"
                      description="Automatically discover and queue new URLs during crawling"
                    />
                  </div>

                  <div className="col-md-6">
                    <Toggle
                      enabled={config.latin_word_filter.exclude_numeric}
                      onChange={(enabled) => handleConfigFieldChange('latin_word_filter.exclude_numeric', enabled)}
                      label="Content Filter"
                      description="Filter out numeric words and apply content filtering rules"
                    />
                  </div>

                  <div className="col-md-6">
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
                </div>
              </div>
            )}
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

      {/* Results Display */}
      <div className="card-google">
        <div className="d-flex align-items-center justify-content-between mb-4">
          <h2 className="google-header h5 mb-0">
            Crawl Results ({crawlResults.length})
          </h2>
          <div className="d-flex gap-2">
            <button
              onClick={onGetResults}
              className="btn btn-google-secondary btn-sm d-flex align-items-center"
            >
              <i className="bi bi-arrow-clockwise me-1"></i>
              Refresh Results
            </button>
            {crawlResults.length > 0 && (
              <button
                onClick={exportResults}
                className="btn btn-google-primary btn-sm d-flex align-items-center"
              >
                <i className="bi bi-download me-1"></i>
                Export JSON
              </button>
            )}
          </div>
        </div>

        {crawlResults.length === 0 ? (
          <div className="text-center py-5 text-muted">
            <i className="bi bi-file-earmark-text display-4 mb-3 d-block"></i>
            <p className="mb-0">No crawl results yet. Start a crawl to see results here.</p>
          </div>
        ) : (
          <div className="overflow-auto" style={{ maxHeight: '400px' }}>
            <div className="d-flex flex-column gap-3">
              {crawlResults.map((result, index) => (
                <div key={index} className="border rounded p-3">
                  <div className="d-flex align-items-start justify-content-between">
                    <div className="flex-grow-1">
                      <h6 className="fw-medium mb-1">
                        {result.title || 'Untitled'}
                      </h6>
                      <a 
                        href={result.url} 
                        target="_blank" 
                        rel="noopener noreferrer"
                        className="text-decoration-none small text-primary d-block mb-2"
                      >
                        {result.url}
                      </a>
                      <p className="small text-muted mb-2" style={{ 
                        display: '-webkit-box',
                        WebkitLineClamp: 2,
                        WebkitBoxOrient: 'vertical',
                        overflow: 'hidden'
                      }}>
                        {result.content}
                      </p>
                      <div className="d-flex justify-content-between align-items-center">
                        <span className={`badge ${
                          result.status_code === 200 ? 'bg-success' : 'bg-warning'
                        }`}>
                          {result.status_code}
                        </span>
                        <small className="text-muted">
                          {formatTimestamp(result.timestamp)}
                        </small>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default CrawlerDashboard
