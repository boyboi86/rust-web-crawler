import { useState, useRef, useEffect } from 'react'

interface Language {
  code: string
  name: string
  flag: string
}

interface LanguageDropdownProps {
  selectedLanguages: string[]
  onChange: (languages: string[]) => void
}

const AVAILABLE_LANGUAGES: Language[] = [
  { code: 'en', name: 'English', flag: '🇺🇸' },
  { code: 'fr', name: 'French', flag: '🇫🇷' },
  { code: 'de', name: 'German', flag: '🇩🇪' },
  { code: 'zh', name: 'Chinese', flag: '🇨🇳' },
  { code: 'ja', name: 'Japanese', flag: '🇯🇵' },
  { code: 'ko', name: 'Korean', flag: '🇰🇷' },
  { code: 'es', name: 'Spanish', flag: '🇪🇸' },
  { code: 'it', name: 'Italian', flag: '🇮🇹' },
  { code: 'pt', name: 'Portuguese', flag: '🇵🇹' },
  { code: 'ru', name: 'Russian', flag: '🇷🇺' },
  { code: 'ar', name: 'Arabic', flag: '🇸🇦' },
  { code: 'hi', name: 'Hindi', flag: '🇮🇳' }
]

function LanguageDropdown({ selectedLanguages, onChange }: LanguageDropdownProps) {
  const [isOpen, setIsOpen] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const toggleLanguage = (langCode: string) => {
    const newSelection = selectedLanguages.includes(langCode)
      ? selectedLanguages.filter(code => code !== langCode)
      : [...selectedLanguages, langCode]
    onChange(newSelection)
  }

  const getSelectedLanguageNames = () => {
    if (selectedLanguages.length === 0) return 'Select languages...'
    if (selectedLanguages.length === 1) {
      const lang = AVAILABLE_LANGUAGES.find(l => l.code === selectedLanguages[0])
      return lang ? `${lang.flag} ${lang.name}` : selectedLanguages[0]
    }
    return `${selectedLanguages.length} languages selected`
  }

  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-900 dark:text-white">
        Accepted Languages
      </label>
      
      <div className="relative" ref={dropdownRef}>
        <button
          type="button"
          onClick={() => setIsOpen(!isOpen)}
          className="relative w-full bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm pl-3 pr-10 py-2 text-left cursor-pointer focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
        >
          <span className="block truncate text-gray-900 dark:text-white">
            {getSelectedLanguageNames()}
          </span>
          <span className="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
            <svg
              className={`h-5 w-5 text-gray-400 transform transition-transform ${
                isOpen ? 'rotate-180' : ''
              }`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </span>
        </button>

        {isOpen && (
          <div className="absolute z-10 mt-1 w-full bg-white dark:bg-gray-700 shadow-lg max-h-60 rounded-md py-1 text-base ring-1 ring-black ring-opacity-5 overflow-auto focus:outline-none sm:text-sm">
            {AVAILABLE_LANGUAGES.map((language) => (
              <div
                key={language.code}
                onClick={() => toggleLanguage(language.code)}
                className="cursor-pointer select-none relative py-2 pl-3 pr-9 hover:bg-blue-50 dark:hover:bg-gray-600"
              >
                <div className="flex items-center">
                  <span className="text-lg mr-3">{language.flag}</span>
                  <span className="font-normal block truncate text-gray-900 dark:text-white">
                    {language.name}
                  </span>
                  <span className="text-gray-500 dark:text-gray-400 ml-2 text-sm">
                    ({language.code})
                  </span>
                </div>
                {selectedLanguages.includes(language.code) && (
                  <span className="absolute inset-y-0 right-0 flex items-center pr-4">
                    <svg
                      className="h-5 w-5 text-blue-600"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fillRule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </span>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {selectedLanguages.length > 0 && (
        <div className="flex flex-wrap gap-2 mt-2">
          {selectedLanguages.map((langCode) => {
            const lang = AVAILABLE_LANGUAGES.find(l => l.code === langCode)
            return (
              <span
                key={langCode}
                className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-800 dark:text-blue-100"
              >
                {lang ? `${lang.flag} ${lang.name}` : langCode}
                <button
                  type="button"
                  onClick={() => toggleLanguage(langCode)}
                  className="ml-1 inline-flex items-center justify-center w-4 h-4 rounded-full hover:bg-blue-200 dark:hover:bg-blue-700"
                >
                  <svg className="w-2 h-2" fill="currentColor" viewBox="0 0 8 8">
                    <path d="M1.41 0l-1.41 1.41.72.72 1.78 1.81-1.78 1.78-.72.69 1.41 1.44.72-.72 1.81-1.81 1.78 1.81.69.72 1.44-1.44-.72-.69-1.81-1.78 1.81-1.81.72-.72-1.44-1.41-.69.72-1.78 1.78-1.81-1.78-.72-.72z" />
                  </svg>
                </button>
              </span>
            )
          })}
        </div>
      )}
    </div>
  )
}

export default LanguageDropdown
