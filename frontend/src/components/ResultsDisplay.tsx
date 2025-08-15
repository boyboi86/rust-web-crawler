interface CrawlResult {
  url: string;
  title: string;
  content: string;
  status_code: number;
  timestamp: string;
}

interface ResultsDisplayProps {
  results: CrawlResult[];
}

function ResultsDisplay({ results }: ResultsDisplayProps) {
  if (results.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
        <h2 className="text-2xl font-semibold mb-4 text-gray-800 dark:text-white">
          Crawl Results
        </h2>
        <p className="text-gray-600 dark:text-gray-400">No results yet. Start a crawl to see data here.</p>
      </div>
    )
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
      <h2 className="text-2xl font-semibold mb-4 text-gray-800 dark:text-white">
        Crawl Results ({results.length})
      </h2>
      
      <div className="space-y-4 max-h-96 overflow-y-auto">
        {results.map((result, index) => (
          <div key={index} className="border dark:border-gray-600 rounded-lg p-4 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
            <div className="flex justify-between items-start mb-2">
              <h3 className="font-semibold text-lg text-gray-800 dark:text-white truncate">
                {result.title || 'Untitled Page'}
              </h3>
              <span className={`px-2 py-1 rounded text-xs font-medium ${
                result.status_code === 200 
                  ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                  : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
              }`}>
                {result.status_code}
              </span>
            </div>
            
            <div className="text-sm text-blue-600 dark:text-blue-400 mb-2 break-all">
              <a href={result.url} target="_blank" rel="noopener noreferrer" className="hover:underline">
                {result.url}
              </a>
            </div>
            
            <p className="text-gray-600 dark:text-gray-300 text-sm line-clamp-3">
              {result.content.substring(0, 200)}
              {result.content.length > 200 && '...'}
            </p>
            
            <div className="text-xs text-gray-500 dark:text-gray-400 mt-2">
              Crawled: {new Date(result.timestamp).toLocaleString()}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

export default ResultsDisplay
