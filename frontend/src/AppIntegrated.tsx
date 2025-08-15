import { useState } from 'react'
import CrawlerDashboardIntegrated from './components/features/CrawlerDashboardIntegrated'
import { CrawlerFormConfig } from './types/crawler'

function App() {
  const [config, setConfig] = useState<CrawlerFormConfig | null>(null)

  const handleConfigChange = (newConfig: CrawlerFormConfig) => {
    setConfig(newConfig)
    console.log('Config updated:', newConfig)
  }

  return (
    <div className="min-vh-100 bg-light">
      {/* Header */}
      <nav className="navbar navbar-expand-lg navbar-dark bg-primary shadow">
        <div className="container">
          <a className="navbar-brand d-flex align-items-center" href="#">
            <i className="bi bi-globe2 me-2 fs-4"></i>
            <span className="fw-bold">Web Crawler</span>
          </a>
          <div className="navbar-nav ms-auto">
            <span className="navbar-text">
              Rust-powered web crawler with Tauri
            </span>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <div className="container py-5">
        <div className="row justify-content-center">
          <div className="col-12 col-lg-10 col-xl-8">
            {/* Title Section */}
            <div className="text-center mb-5">
              <h1 className="display-5 fw-bold text-dark mb-3">
                Web Crawler Dashboard
              </h1>
              <p className="lead text-muted">
                Configure and execute web crawling sessions with advanced filtering options
              </p>
            </div>

            {/* Dashboard Component */}
            <CrawlerDashboardIntegrated onConfigChange={handleConfigChange} />

            {/* Debug Info (Development Only) */}
            {config && (
              <div className="mt-5">
                <div className="card bg-dark text-light">
                  <div className="card-header">
                    <h6 className="mb-0">Debug: Current Configuration</h6>
                  </div>
                  <div className="card-body">
                    <pre className="small mb-0">
                      {JSON.stringify(config, null, 2)}
                    </pre>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Footer */}
      <footer className="bg-dark text-light py-4 mt-5">
        <div className="container">
          <div className="row align-items-center">
            <div className="col-md-6">
              <small className="text-muted">
                Built with React, TypeScript, Tauri, and Rust
              </small>
            </div>
            <div className="col-md-6 text-md-end">
              <small className="text-muted">
                Desktop Web Crawler v1.0
              </small>
            </div>
          </div>
        </div>
      </footer>
    </div>
  )
}

export default App
