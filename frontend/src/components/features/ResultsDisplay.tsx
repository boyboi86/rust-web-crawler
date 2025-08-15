import { Card, StatusBadge } from '../ui'

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
      <Card title="Crawl Results">
        <p className="text-muted">No results yet. Start a crawl to see data here.</p>
      </Card>
    )
  }

  return (
    <Card title={`Crawl Results (${results.length})`}>
      <div className="space-y-4 max-h-96 overflow-y-auto">
        {results.map((result, index) => (
          <div key={index} className="border rounded-lg p-4 hover:bg-gray-50 transition-colors">
            <div className="d-flex justify-content-between align-items-start mb-2">
              <h3 className="google-subheader fw-medium text-truncate">
                {result.title || 'Untitled Page'}
              </h3>
              <StatusBadge 
                status={result.status_code === 200 ? 'success' : 'danger'}
                className="ms-2"
              >
                {result.status_code}
              </StatusBadge>
            </div>
            
            <div className="text-primary mb-2 text-break">
              <a href={result.url} target="_blank" rel="noopener noreferrer" className="text-decoration-none">
                <i className="bi bi-link-45deg me-1"></i>
                {result.url}
              </a>
            </div>
            
            <p className="text-muted small">
              {result.content.substring(0, 200)}
              {result.content.length > 200 && '...'}
            </p>
            
            <div className="text-muted small mt-2">
              <i className="bi bi-clock me-1"></i>
              Crawled: {new Date(result.timestamp).toLocaleString()}
            </div>
          </div>
        ))}
      </div>
    </Card>
  )
}

export default ResultsDisplay
