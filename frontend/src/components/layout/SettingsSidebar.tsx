import { useState } from 'react'
import { WebCrawlerConfig } from '../../App'
import { Toggle, MaterialInput } from '../ui'
import { NumberInput } from '../forms'

interface SettingsSidebarProps {
  isOpen: boolean
  onClose: () => void
  config: WebCrawlerConfig
  onConfigChange: (config: WebCrawlerConfig) => void
  onApplyPreset: (presetName: string) => void
}

const AVAILABLE_PRESETS = [
  { name: 'production', label: 'Production', description: 'Optimized for production crawling' },
  { name: 'development', label: 'Development', description: 'For development and testing' },
  { name: 'demo', label: 'Demo', description: 'Demonstration configuration' }
]

function SettingsSidebar({ isOpen, onClose, config, onConfigChange, onApplyPreset }: SettingsSidebarProps) {
  const [activeSection, setActiveSection] = useState<'presets' | 'general' | 'filtering' | 'advanced'>('presets')

  const handleInputChange = (field: string, value: any) => {
    const newConfig = { ...config }
    
    if (field.includes('.')) {
      const [parent, child] = field.split('.')
      if (parent === 'latin_word_filter') {
        newConfig.latin_word_filter = {
          ...newConfig.latin_word_filter,
          [child]: value
        }
      }
    } else {
      (newConfig as any)[field] = value
    }
    
    onConfigChange(newConfig)
  }

  const renderPresetsSection = () => (
    <div className="d-flex flex-column gap-4">
      <h3 className="google-header h5">Configuration Presets</h3>
      <div className="d-flex flex-column gap-3">
        {AVAILABLE_PRESETS.map((preset) => (
          <div key={preset.name} className="card">
            <div className="card-body p-3">
              <div className="d-flex justify-content-between align-items-start">
                <div className="flex-grow-1">
                  <h6 className="card-title mb-1">{preset.label}</h6>
                  <p className="card-text small text-muted mb-2">{preset.description}</p>
                </div>
                <button
                  onClick={() => onApplyPreset(preset.name)}
                  className="btn btn-google-primary btn-sm"
                >
                  Apply
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  )

  const renderGeneralSection = () => (
    <div className="d-flex flex-column gap-4">
      <h3 className="google-header h5">General Settings</h3>
      
      <div className="row g-4">
        <div className="col-12">
          <NumberInput
            id="maxCrawlDepth"
            label="Max Crawl Depth"
            value={config.max_crawl_depth}
            onChange={(value) => handleInputChange('max_crawl_depth', value)}
            min={1}
            max={10}
          />
        </div>
        
        <div className="col-12">
          <NumberInput
            id="maxTotalUrls"
            label="Max Total URLs"
            value={config.max_total_urls}
            onChange={(value) => handleInputChange('max_total_urls', value)}
            min={1}
            max={10000}
          />
        </div>

        <div className="col-12">
          <MaterialInput
            id="userAgent"
            label="User Agent"
            value={config.user_agent}
            onChange={(value) => handleInputChange('user_agent', value)}
            type="text"
          />
        </div>
      </div>
    </div>
  )

  const renderFilteringSection = () => (
    <div className="d-flex flex-column gap-4">
      <h3 className="google-header h5">Content Filtering</h3>
      
      <div className="row g-4">
        <div className="col-12">
          <MaterialInput
            id="filterTargetWords"
            label="Target Words (comma-separated)"
            value={config.target_words.join(', ')}
            onChange={(value) => handleInputChange('target_words', value.split(',').map(w => w.trim()).filter(w => w))}
            type="text"
          />
        </div>

        <div className="col-12">
          <NumberInput
            id="minWordLength"
            label="Minimum Word Length"
            value={config.min_word_length}
            onChange={(value) => handleInputChange('min_word_length', value)}
            min={1}
            max={50}
          />
        </div>

        <div className="col-12">
          <MaterialInput
            id="excludedWords"
            label="Excluded Words (comma-separated)"
            value={config.latin_word_filter.excluded_words.join(', ')}
            onChange={(value) => handleInputChange('latin_word_filter.excluded_words', value.split(',').map(w => w.trim()).filter(w => w))}
            type="text"
          />
        </div>

        <div className="col-12">
          <div className="material-group">
            <select
              value={config.accepted_languages.length > 0 ? config.accepted_languages[0] : ''}
              onChange={(e) => {
                const value = e.target.value
                if (value) {
                  handleInputChange('accepted_languages', [value])
                } else {
                  handleInputChange('accepted_languages', [])
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
            <label className="material-label">Accepted Language</label>
            <span className="material-bar"></span>
          </div>
        </div>
      </div>
    </div>
  )

  const renderAdvancedSection = () => (
    <div className="d-flex flex-column gap-5">
      <h3 className="google-header h5">Advanced Settings</h3>
      
      {/* Proxy Configuration */}
      <div className="bg-transparent">
        <h6 className="google-subheader mb-4">Proxy Configuration</h6>
        <div>
          <label className="form-label google-text mb-2">Proxy Pool (one per line)</label>
          <textarea
            value={config.proxy_pool.join('\n')}
            onChange={(e) => handleInputChange('proxy_pool', e.target.value.split('\n').filter(line => line.trim()))}
            placeholder="http://proxy1.example.com:8080&#10;http://proxy2.example.com:8080"
            rows={3}
            className="form-control border-0 border-bottom rounded-0 bg-transparent"
            style={{ borderBottomWidth: '1px !important' }}
          />
        </div>
      </div>

      {/* Rate Limiting */}
      <div className="bg-transparent">
        <h6 className="google-subheader mb-4">Rate Limiting</h6>
        <div className="d-flex flex-column gap-4">
          <NumberInput
            id="maxRequests"
            label="Max Requests/sec"
            value={config.max_requests_per_second}
            onChange={(value) => handleInputChange('max_requests_per_second', value)}
            min={1}
            max={100}
          />
          <NumberInput
            id="windowSize"
            label="Window Size (ms)"
            value={config.window_size_ms}
            onChange={(value) => handleInputChange('window_size_ms', value)}
            min={100}
            max={10000}
          />
        </div>
      </div>

      {/* Retry Configuration */}
      <div className="bg-transparent">
        <h6 className="google-subheader mb-4">Retry Configuration</h6>
        <div className="d-flex flex-column gap-4">
          <NumberInput
            id="maxRetries"
            label="Max Retries"
            value={config.max_retries}
            onChange={(value) => handleInputChange('max_retries', value)}
            min={0}
            max={10}
          />
          <NumberInput
            id="baseDelay"
            label="Base Delay (ms)"
            value={config.base_delay_ms}
            onChange={(value) => handleInputChange('base_delay_ms', value)}
            min={100}
            max={10000}
          />
        </div>
      </div>

      {/* Data Output */}
      <div className="bg-transparent">
        <h6 className="google-subheader mb-4">Data Output</h6>
        <div className="d-flex flex-column gap-4">
          <NumberInput
            id="maxFileSize"
            label="Max File Size (MB)"
            value={config.max_file_size_mb}
            onChange={(value) => handleInputChange('max_file_size_mb', value)}
            min={1}
            max={1000}
          />
          <div>
            <label className="form-label google-text mb-2">Output Format</label>
            <select
              value={config.data_output_extension}
              onChange={(e) => handleInputChange('data_output_extension', e.target.value)}
              className="form-select border-0 border-bottom rounded-0 bg-transparent"
              style={{ borderBottomWidth: '1px !important' }}
            >
              <option value="json">JSON</option>
              <option value="csv">CSV</option>
            </select>
          </div>
        </div>
      </div>

      {/* Monitoring */}
      <div className="bg-transparent">
        <h6 className="google-subheader mb-4">Monitoring</h6>
        <div className="d-flex flex-column gap-4">
          <Toggle
            enabled={config.enable_metrics}
            onChange={(enabled) => handleInputChange('enable_metrics', enabled)}
            label="Enable Metrics"
            description="Enable metrics collection"
          />
          
          <Toggle
            enabled={config.persist_queue}
            onChange={(enabled) => handleInputChange('persist_queue', enabled)}
            label="Persist Queue"
            description="Save queue state to disk"
          />
        </div>
      </div>
    </div>
  )

  return (
    <div className={`settings-sidebar-enhanced ${isOpen ? 'open' : ''}`}>
      {/* Mobile Header */}
      <div className="d-lg-none border-bottom p-3 bg-white">
        <div className="d-flex align-items-center justify-content-between">
          <h5 className="google-header mb-0">Settings</h5>
          <button
            onClick={onClose}
            className="btn btn-link p-0"
            type="button"
          >
            <i className="bi bi-x-lg"></i>
          </button>
        </div>
      </div>

      <div className="p-3 h-100 overflow-auto">
        {/* Navigation Pills */}
        <div className="mb-4">
          <ul className="nav nav-pills nav-fill flex-column flex-sm-row" role="tablist">
            <li className="nav-item" role="presentation">
              <button
                className={`nav-link ${activeSection === 'presets' ? 'active' : ''}`}
                onClick={() => setActiveSection('presets')}
                type="button"
              >
                <i className="bi bi-gear me-1"></i>
                <span className="d-none d-sm-inline">Presets</span>
              </button>
            </li>
            <li className="nav-item" role="presentation">
              <button
                className={`nav-link ${activeSection === 'general' ? 'active' : ''}`}
                onClick={() => setActiveSection('general')}
                type="button"
              >
                <i className="bi bi-sliders me-1"></i>
                <span className="d-none d-sm-inline">General</span>
              </button>
            </li>
            <li className="nav-item" role="presentation">
              <button
                className={`nav-link ${activeSection === 'filtering' ? 'active' : ''}`}
                onClick={() => setActiveSection('filtering')}
                type="button"
              >
                <i className="bi bi-funnel me-1"></i>
                <span className="d-none d-sm-inline">Filtering</span>
              </button>
            </li>
            <li className="nav-item" role="presentation">
              <button
                className={`nav-link ${activeSection === 'advanced' ? 'active' : ''}`}
                onClick={() => setActiveSection('advanced')}
                type="button"
              >
                <i className="bi bi-gear-wide-connected me-1"></i>
                <span className="d-none d-sm-inline">Advanced</span>
              </button>
            </li>
          </ul>
        </div>

        {/* Content Sections */}
        <div className="tab-content">
          {activeSection === 'presets' && renderPresetsSection()}
          {activeSection === 'general' && renderGeneralSection()}
          {activeSection === 'filtering' && renderFilteringSection()}
          {activeSection === 'advanced' && renderAdvancedSection()}
        </div>
      </div>
    </div>
  )
}

export default SettingsSidebar
