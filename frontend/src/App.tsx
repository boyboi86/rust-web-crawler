import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import CrawlerForm from './components/CrawlerForm'
import StatusDisplay from './components/StatusDisplay'
import ResultsDisplay from './components/ResultsDisplay'

interface CrawlResult {
  url: string;
  title: string;
  content: string;
  status_code: number;
  timestamp: string;
}

interface CrawlStatus {
  total_pages: number;
  processed_pages: number;
  queue_size: number;
  is_running: boolean;
}

function App() {
  const [crawlResults, setCrawlResults] = useState<CrawlResult[]>([])
  const [crawlStatus, setCrawlStatus] = useState<CrawlStatus | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  useEffect(() => {
    // Poll for status updates when crawling is active
    let interval: number | null = null;
    
    if (crawlStatus?.is_running) {
      interval = setInterval(async () => {
        try {
          const status: CrawlStatus = await invoke('get_crawl_status')
          setCrawlStatus(status)
        } catch (error) {
          console.error('Failed to fetch status:', error)
        }
      }, 1000)
    }

    return () => {
      if (interval) clearInterval(interval)
    }
  }, [crawlStatus?.is_running])

  const handleStartCrawl = async (startUrl: string, maxPages: number) => {
    setIsLoading(true)
    setCrawlResults([])
    
    try {
      await invoke('start_crawl', { startUrl, maxPages })
      
      // Start polling for status
      const status: CrawlStatus = await invoke('get_crawl_status')
      setCrawlStatus(status)
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
      console.error('Failed to stop crawl:', error)
    }
  }

  const handleGetResults = async () => {
    try {
      const results: CrawlResult[] = await invoke('get_crawl_results')
      setCrawlResults(results)
    } catch (error) {
      console.error('Failed to get results:', error)
    }
  }

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900">
      <div className="container mx-auto px-4 py-8">
        <h1 className="text-4xl font-bold text-center mb-8 text-gray-800 dark:text-white">
          Rust Web Crawler
        </h1>
        
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Crawler Controls */}
          <div className="lg:col-span-1">
            <CrawlerForm 
              onStartCrawl={handleStartCrawl}
              onStopCrawl={handleStopCrawl}
              isLoading={isLoading}
              isRunning={crawlStatus?.is_running || false}
            />
          </div>
          
          {/* Status and Results */}
          <div className="lg:col-span-2 space-y-6">
            <StatusDisplay 
              status={crawlStatus}
              onGetResults={handleGetResults}
            />
            
            <ResultsDisplay results={crawlResults} />
          </div>
        </div>
      </div>
    </div>
  )
}

export default App
