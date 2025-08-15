import { useState } from 'react'
import { WebCrawlerConfig } from '../App'
import Toggle from './Toggle'
import LanguageDropdown from './LanguageDropdown'

interface ConfigurationPanelProps {
  config: WebCrawlerConfig
  onConfigChange: (config: WebCrawlerConfig) => void
  onApplyPreset: (presetName: string) => void
}

const PRESET_CONFIGS = [
  {
    name: 'production',
    label: 'Production',
    description: 'Optimized for production crawling with comprehensive settings',
    icon: '🏭',
    details: 'High depth, extensive crawling, language detection enabled'
  },
  {
    name: 'development',
    label: 'Development',
    description: 'Development configuration for testing and debugging',
    icon: '🛠️',
    details: 'Medium depth, keyword filtering, development-friendly settings'
  },
  {
    name: 'demo',
    label: 'Demo',
    description: 'Quick demonstration configuration with limited scope',
    icon: '🎯',
    details: 'Limited depth, fast crawling, demonstration purposes'
  }
]

function ConfigurationPanel({ config, onConfigChange, onApplyPreset }: ConfigurationPanelProps) {
  const [activeTab, setActiveTab] = useState<'presets' | 'manual'>('presets')
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})

  const validateField = (field: string, value: any): string => {
    switch (field) {
      case 'user_agent':
        if (!value || value.trim().length < 5) {
          return 'User agent must be at least 5 characters long'
        }
        if (value.length > 200) {
          return 'User agent must be less than 200 characters'
        }
        break
      case 'max_crawl_depth':
        if (value < 1 || value > 10) {
          return 'Crawl depth must be between 1 and 10'
        }
        break
      case 'max_total_urls':
        if (value < 1 || value > 10000) {
          return 'Max total URLs must be between 1 and 10,000'
        }
        break
      case 'min_word_length':
        if (value < 1 || value > 50) {
          return 'Min word length must be between 1 and 50'
        }
        break
    }
    return ''
  }

  const handleInputChange = (field: string, value: any) => {
    // Validate the field
    const error = validateField(field, value)
    setValidationErrors(prev => ({
      ...prev,
      [field]: error
    }))

    // Sanitize the value
    let sanitizedValue = value
    if (typeof value === 'string') {
      sanitizedValue = value.trim().replace(/[<>'"&]/g, '').substring(0, 500)
    }

    // Update config
    const newConfig = { ...config }
    if (field.includes('.')) {
      const [parent, child] = field.split('.')
      if (parent === 'latin_word_filter') {
        newConfig.latin_word_filter = {
          ...newConfig.latin_word_filter,
          [child]: sanitizedValue
        }
      }
    } else {
      (newConfig as any)[field] = sanitizedValue
    }
    
    onConfigChange(newConfig)
  }

  const handleArrayChange = (field: 'accepted_languages' | 'target_words' | 'avoid_url_extensions', value: string) => {
    const sanitizedValue = value.replace(/[<>'"&]/g, '')
    const array = sanitizedValue.split(',').map(item => item.trim()).filter(item => item)
    
    const newConfig = { ...config }
    newConfig[field] = array
    onConfigChange(newConfig)
  }

  const renderPresetCard = (preset: typeof PRESET_CONFIGS[0]) => (
    <div key={preset.name} className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-6 hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between">
        <div className="flex items-start space-x-4">
          <div className="text-3xl">{preset.icon}</div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {preset.label}
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              {preset.description}
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-500 mt-2">
              {preset.details}
            </p>
          </div>
        </div>
        <button
          onClick={() => onApplyPreset(preset.name)}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
        >
          Apply
        </button>
      </div>
    </div>
  )

  const renderFormField = (
    label: string,
    field: string,
    type: 'text' | 'number' | 'toggle' | 'textarea' | 'languages' = 'text',
    options?: { min?: number; max?: number; placeholder?: string; description?: string }
  ) => {
    const value = field.includes('.') 
      ? config.latin_word_filter[field.split('.')[1] as keyof typeof config.latin_word_filter]
      : (config as any)[field]

    const error = validationErrors[field]

    return (
      <div className="space-y-2">
        {type === 'toggle' ? (
          <Toggle
            enabled={Boolean(value)}
            onChange={(enabled) => handleInputChange(field, enabled)}
            label={label}
            description={options?.description}
          />
        ) : type === 'languages' ? (
          <LanguageDropdown
            selectedLanguages={config.accepted_languages}
            onChange={(languages) => handleInputChange('accepted_languages', languages)}
          />
        ) : (
          <>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              {label}
            </label>
            {type === 'textarea' ? (
              <textarea
                value={Array.isArray(value) ? value.join(', ') : String(value)}
                onChange={(e) => field.includes('_words') || field.includes('_url') || field.includes('_extensions') || field.includes('_languages')
                  ? handleArrayChange(field as any, e.target.value)
                  : handleInputChange(field, e.target.value)
                }
                rows={3}
                placeholder={options?.placeholder}
                className={`w-full px-3 py-2 border rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white ${
                  error ? 'border-red-300' : 'border-gray-300'
                }`}
              />
            ) : (
              <input
                type={type}
                value={type === 'number' ? Number(value) : String(value)}
                onChange={(e) => handleInputChange(
                  field, 
                  type === 'number' ? parseInt(e.target.value) || 0 : e.target.value
                )}
                min={options?.min}
                max={options?.max}
                placeholder={options?.placeholder}
                className={`w-full px-3 py-2 border rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white ${
                  error ? 'border-red-300' : 'border-gray-300'
                }`}
              />
            )}
            {error && (
              <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
            )}
          </>
        )}
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header with tabs */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Configuration Management
          </h2>
          <div className="flex space-x-1 bg-gray-100 dark:bg-gray-700 p-1 rounded-lg">
            <button
              onClick={() => setActiveTab('presets')}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'presets'
                  ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow'
                  : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
              }`}
            >
              Presets
            </button>
            <button
              onClick={() => setActiveTab('manual')}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'manual'
                  ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow'
                  : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
              }`}
            >
              Manual Configuration
            </button>
          </div>
        </div>

        {activeTab === 'presets' ? (
          <div className="space-y-4">
            <p className="text-gray-600 dark:text-gray-400">
              Choose from predefined configurations optimized for different use cases.
            </p>
            <div className="grid gap-4">
              {PRESET_CONFIGS.map(renderPresetCard)}
            </div>
          </div>
        ) : (
          <div className="space-y-6">
            <p className="text-gray-600 dark:text-gray-400">
              Manually configure all crawler settings. All inputs are sanitized to prevent injection attacks.
            </p>

            {/* General Settings */}
            <div className="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">General Settings</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {renderFormField('User Agent', 'user_agent', 'text', { 
                  placeholder: 'Mozilla/5.0 (compatible; WebCrawler/1.0)' 
                })}
                {renderFormField('Max Crawl Depth', 'max_crawl_depth', 'number', { min: 1, max: 10 })}
                {renderFormField('Max Total URLs', 'max_total_urls', 'number', { min: 1, max: 10000 })}
                {renderFormField('Min Word Length', 'min_word_length', 'number', { min: 1, max: 50 })}
              </div>
            </div>

            {/* URL Settings */}
            <div className="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">URL Settings</h3>
              <div className="space-y-4">
                {renderFormField('Avoid URL Extensions (comma-separated)', 'avoid_url_extensions', 'textarea', { 
                  placeholder: '.pdf, .doc, .zip, .exe' 
                })}
              </div>
            </div>

            {/* Content Filtering */}
            <div className="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Content Filtering</h3>
              <div className="space-y-4">
                {renderFormField('Keyword Filtering', 'enable_keyword_filtering', 'toggle', {
                  description: 'Only crawl pages containing target keywords'
                })}
                {renderFormField('Target Keywords (comma-separated)', 'target_words', 'textarea', { 
                  placeholder: 'technology, innovation, software, programming' 
                })}
                {renderFormField('Accepted Languages', 'accepted_languages', 'languages')}
              </div>
            </div>

            {/* Advanced Settings */}
            <div className="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Advanced Settings</h3>
              <div className="space-y-4">
                {renderFormField('Extension Crawling', 'enable_extension_crawling', 'toggle', {
                  description: 'Automatically discover and queue new URLs during crawling'
                })}
                
                <div className="border-t border-gray-200 dark:border-gray-600 pt-4">
                  <h4 className="text-md font-medium text-gray-900 dark:text-white mb-3">Latin Word Filter</h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {renderFormField('Exclude Numeric Words', 'latin_word_filter.exclude_numeric', 'toggle', {
                      description: 'Filter out words containing numbers'
                    })}
                    {renderFormField('Filter Min Word Length', 'latin_word_filter.min_word_length', 'number', { min: 1, max: 50 })}
                  </div>
                  <div className="mt-4">
                    {renderFormField('Excluded Words (comma-separated)', 'latin_word_filter.excluded_words', 'textarea', { 
                      placeholder: 'the, and, or, but, in, on, at' 
                    })}
                  </div>
                </div>
              </div>
            </div>

            {/* Configuration Preview */}
            <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Configuration Preview</h3>
              <pre className="text-sm bg-white dark:bg-gray-800 p-4 rounded border overflow-x-auto">
                {JSON.stringify(config, null, 2)}
              </pre>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default ConfigurationPanel
