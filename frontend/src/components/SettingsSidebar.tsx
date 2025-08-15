import { useState } from 'react'
import { WebCrawlerConfig } from '../App'
import Toggle from './Toggle'
import LanguageDropdown from './LanguageDropdown'

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

function SettingsSidebar({ onClose, config, onConfigChange, onApplyPreset }: SettingsSidebarProps) {
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
      
      <div className="row g-3">
        <div className="col-12">
          <label className="form-label google-subheader">Max Crawl Depth</label>
          <input
            type="number"
            min="1"
            max="10"
            value={config.max_crawl_depth}
            onChange={(e) => handleInputChange('max_crawl_depth', parseInt(e.target.value) || 1)}
            className="form-control"
          />
        </div>
        
        <div className="col-12">
          <label className="form-label google-subheader">Max Total URLs</label>
          <input
            type="number"
            min="1"
            max="10000"
            value={config.max_total_urls}
            onChange={(e) => handleInputChange('max_total_urls', parseInt(e.target.value) || 1)}
            className="form-control"
          />
        </div>

        <div className="col-12">
          <div className="form-group-material">
            <input
              type="text"
              value={config.user_agent}
              onChange={(e) => handleInputChange('user_agent', e.target.value)}
              placeholder=" "
              className="form-control-material"
              id="userAgent"
            />
            <span className="material-bar"></span>
            <label htmlFor="userAgent" className="form-label">User Agent</label>
          </div>
        </div>

        <div className="col-12">
          <label className="form-label google-subheader">Accepted Languages</label>
          <LanguageDropdown
            selectedLanguages={config.accepted_languages}
            onChange={(languages) => handleInputChange('accepted_languages', languages)}
          />
        </div>
      </div>
    </div>
  )

  const renderFilteringSection = () => (
    <div className="d-flex flex-column gap-4">
      <h3 className="google-header h5">Content Filtering</h3>
      
      <div className="row g-3">
        <div className="col-12">
          <div className="form-group-material">
            <input
              type="text"
              value={config.target_words.join(', ')}
              onChange={(e) => handleInputChange('target_words', e.target.value.split(',').map(w => w.trim()).filter(w => w))}
              placeholder=" "
              className="form-control-material"
              id="filterTargetWords"
            />
            <span className="material-bar"></span>
            <label htmlFor="filterTargetWords" className="form-label">Target Words (comma-separated)</label>
          </div>
        </div>

        <div className="col-12">
          <label className="form-label google-subheader">Minimum Word Length</label>
          <input
            type="number"
            min="1"
            max="50"
            value={config.min_word_length}
            onChange={(e) => handleInputChange('min_word_length', parseInt(e.target.value) || 1)}
            className="form-control"
          />
        </div>

        <div className="col-12">
          <label className="form-label google-subheader">Excluded Words (comma-separated)</label>
          <input
            type="text"
            value={config.latin_word_filter.excluded_words.join(', ')}
            onChange={(e) => handleInputChange('latin_word_filter.excluded_words', e.target.value.split(',').map(w => w.trim()).filter(w => w))}
            placeholder="the, and, or, but"
            className="form-control"
          />
        </div>
      </div>
    </div>
  )

  const renderAdvancedSection = () => (
    <div className="d-flex flex-column gap-4">
      <h3 className="google-header h5">Advanced Settings</h3>
      
      {/* Proxy Configuration */}
      <div className="card bg-light">
        <div className="card-body p-3">
          <h6 className="card-title google-subheader">Proxy Configuration</h6>
          <div>
            <label className="form-label small">Proxy Pool (one per line)</label>
            <textarea
              value={config.proxy_pool.join('\n')}
              onChange={(e) => handleInputChange('proxy_pool', e.target.value.split('\n').filter(line => line.trim()))}
              placeholder="http://proxy1.example.com:8080&#10;http://proxy2.example.com:8080"
              rows={3}
              className="form-control"
            />
          </div>
        </div>
      </div>

      {/* Rate Limiting */}
      <div className="card bg-light">
        <div className="card-body p-3">
          <h6 className="card-title google-subheader">Rate Limiting</h6>
          <div className="row g-2">
            <div className="col-6">
              <label className="form-label small">Max Requests/sec</label>
              <input
                type="number"
                min="1"
                max="100"
                value={config.max_requests_per_second}
                onChange={(e) => handleInputChange('max_requests_per_second', parseInt(e.target.value) || 1)}
                className="form-control form-control-sm"
              />
            </div>
            <div className="col-6">
              <label className="form-label small">Window Size (ms)</label>
              <input
                type="number"
                min="100"
                max="10000"
                value={config.window_size_ms}
                onChange={(e) => handleInputChange('window_size_ms', parseInt(e.target.value) || 1000)}
                className="form-control form-control-sm"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Retry Configuration */}
      <div className="card bg-light">
        <div className="card-body p-3">
          <h6 className="card-title google-subheader">Retry Configuration</h6>
          <div className="row g-2">
            <div className="col-6">
              <label className="form-label small">Max Retries</label>
              <input
                type="number"
                min="0"
                max="10"
                value={config.max_retries}
                onChange={(e) => handleInputChange('max_retries', parseInt(e.target.value) || 0)}
                className="form-control form-control-sm"
              />
            </div>
            <div className="col-6">
              <label className="form-label small">Base Delay (ms)</label>
              <input
                type="number"
                min="100"
                max="10000"
                value={config.base_delay_ms}
                onChange={(e) => handleInputChange('base_delay_ms', parseInt(e.target.value) || 1000)}
                className="form-control form-control-sm"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Data Output */}
      <div className="card bg-light">
        <div className="card-body p-3">
          <h6 className="card-title google-subheader">Data Output</h6>
          <div className="row g-2">
            <div className="col-6">
              <label className="form-label small">Max File Size (MB)</label>
              <input
                type="number"
                min="1"
                max="1000"
                value={config.max_file_size_mb}
                onChange={(e) => handleInputChange('max_file_size_mb', parseInt(e.target.value) || 100)}
                className="form-control form-control-sm"
              />
            </div>
            <div className="col-6">
              <label className="form-label small">Output Format</label>
              <select
                value={config.data_output_extension}
                onChange={(e) => handleInputChange('data_output_extension', e.target.value as 'csv' | 'json')}
                className="form-select form-select-sm"
              >
                <option value="json">JSON</option>
                <option value="csv">CSV</option>
              </select>
            </div>
          </div>
        </div>
      </div>

      {/* Monitoring */}
      <div className="card bg-light">
        <div className="card-body p-3">
          <h6 className="card-title google-subheader">Monitoring</h6>
          <div className="d-flex flex-column gap-2">
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
    </div>
  )

  return (
    <div className="h-100">
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
