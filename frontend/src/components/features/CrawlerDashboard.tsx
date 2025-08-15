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
    if (startUrl.trim()) {
      onStartCrawl(startUrl.trim(), config)
    }
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
                label="Enter URL to crawl"
                value={startUrl}
                onChange={setStartUrl}
                type="url"
                required
                className="primary-url-input"
              />
            </div>
          </div>

          {/* Target Keywords */}
          <div className="mb-4">
            <MaterialInput
              id="targetKeywords"
              label="Target Keywords (comma-separated)"
              value={config.target_words.join(', ')}
              onChange={(value) => handleConfigFieldChange('target_words', value)}
              type="text"
            />
          </div>

          {/* Essential Behavior Toggles */}
          <div className="d-flex flex-column gap-4 mb-4">
            <Toggle
              enabled={config.enable_keyword_filtering}
              onChange={(enabled) => handleConfigFieldChange('enable_keyword_filtering', enabled)}
              label="Target Keyword Filter"
              description="Only crawl pages containing target keywords"
            />

            <Toggle
              enabled={config.enable_extension_crawling}
              onChange={(enabled) => handleConfigFieldChange('enable_extension_crawling', enabled)}
              label="Extension Crawling"
              description="Automatically discover and queue new URLs during crawling"
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
