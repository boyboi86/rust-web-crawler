interface CrawlStatus {
  total_pages: number;
  processed_pages: number;
  queue_size: number;
  is_running: boolean;
}

interface StatusDisplayProps {
  status: CrawlStatus | null;
  onGetResults: () => void;
}

function StatusDisplay({ status, onGetResults }: StatusDisplayProps) {
  if (!status) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
        <h2 className="text-2xl font-semibold mb-4 text-gray-800 dark:text-white">
          Crawler Status
        </h2>
        <p className="text-gray-600 dark:text-gray-400">No crawl session active</p>
      </div>
    )
  }

  const progress = status.total_pages > 0 ? (status.processed_pages / status.total_pages) * 100 : 0

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-2xl font-semibold text-gray-800 dark:text-white">
          Crawler Status
        </h2>
        <div className={`px-3 py-1 rounded-full text-sm font-medium ${
          status.is_running 
            ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' 
            : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'
        }`}>
          {status.is_running ? 'Running' : 'Stopped'}
        </div>
      </div>
      
      <div className="space-y-4">
        <div>
          <div className="flex justify-between text-sm text-gray-600 dark:text-gray-400 mb-1">
            <span>Progress</span>
            <span>{status.processed_pages} / {status.total_pages} pages</span>
          </div>
          <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div 
              className="bg-blue-600 h-2 rounded-full transition-all duration-300"
              style={{ width: `${progress}%` }}
            ></div>
          </div>
        </div>
        
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="bg-gray-50 dark:bg-gray-700 p-3 rounded">
            <div className="text-gray-600 dark:text-gray-400">Queue Size</div>
            <div className="text-xl font-semibold text-gray-800 dark:text-white">
              {status.queue_size}
            </div>
          </div>
          <div className="bg-gray-50 dark:bg-gray-700 p-3 rounded">
            <div className="text-gray-600 dark:text-gray-400">Processed</div>
            <div className="text-xl font-semibold text-gray-800 dark:text-white">
              {status.processed_pages}
            </div>
          </div>
        </div>
        
        <button
          onClick={onGetResults}
          className="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-4 rounded-md transition-colors duration-200"
        >
          Get Results
        </button>
      </div>
    </div>
  )
}

export default StatusDisplay
