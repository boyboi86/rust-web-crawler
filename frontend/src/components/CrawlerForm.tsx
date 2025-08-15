import { useState } from 'react'

interface CrawlerFormProps {
  onStartCrawl: (startUrl: string, maxPages: number) => void;
  onStopCrawl: () => void;
  isLoading: boolean;
  isRunning: boolean;
}

function CrawlerForm({ onStartCrawl, onStopCrawl, isLoading, isRunning }: CrawlerFormProps) {
  const [startUrl, setStartUrl] = useState('https://example.com')
  const [maxPages, setMaxPages] = useState(10)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!isRunning && !isLoading) {
      onStartCrawl(startUrl, maxPages)
    }
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
      <h2 className="text-2xl font-semibold mb-4 text-gray-800 dark:text-white">
        Crawler Configuration
      </h2>
      
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label htmlFor="startUrl" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Starting URL
          </label>
          <input
            type="url"
            id="startUrl"
            value={startUrl}
            onChange={(e) => setStartUrl(e.target.value)}
            disabled={isRunning || isLoading}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            placeholder="https://example.com"
            required
          />
        </div>
        
        <div>
          <label htmlFor="maxPages" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Maximum Pages
          </label>
          <input
            type="number"
            id="maxPages"
            value={maxPages}
            onChange={(e) => setMaxPages(parseInt(e.target.value))}
            disabled={isRunning || isLoading}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            min="1"
            max="1000"
            required
          />
        </div>
        
        <div className="flex space-x-3">
          <button
            type="submit"
            disabled={isRunning || isLoading}
            className="flex-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-medium py-2 px-4 rounded-md transition-colors duration-200"
          >
            {isLoading ? 'Starting...' : isRunning ? 'Running' : 'Start Crawl'}
          </button>
          
          {isRunning && (
            <button
              type="button"
              onClick={onStopCrawl}
              className="flex-1 bg-red-600 hover:bg-red-700 text-white font-medium py-2 px-4 rounded-md transition-colors duration-200"
            >
              Stop Crawl
            </button>
          )}
        </div>
      </form>
    </div>
  )
}

export default CrawlerForm
