// =============================================================================
// CRAWLER ACTOR - Thread-Safe Bridge Pattern
// =============================================================================
// This module implements an actor pattern to bridge between Tauri's Send-safe
// environment and the WebCrawler's non-Send types (ThreadRng).
//
// Architecture:
// - CrawlerActor: Owns the WebCrawler in a dedicated thread
// - CrawlerBridge: Send-safe interface for Tauri commands
// - Message Queue: Communication channel between bridge and actor

use std::collections::HashMap;
use std::thread;
use tokio::sync::{mpsc, oneshot};

use crate::core::{CrawlRequest, CrawlResultSummary, CrawlStatus};
use rust_web_crawler::config::WebCrawlerConfig;
use rust_web_crawler::crawler::WebCrawler;

/// Messages sent to the crawler actor
#[derive(Debug)]
pub enum ActorMessage {
    /// Start a new crawl session
    StartCrawl {
        session_id: String,
        request: CrawlRequest,
        response: oneshot::Sender<Result<String, String>>,
    },
    /// Get status of an active crawl session
    GetStatus {
        session_id: String,
        response: oneshot::Sender<Option<CrawlStatus>>,
    },
    /// Stop a crawl session
    StopCrawl {
        session_id: String,
        response: oneshot::Sender<Result<String, String>>,
    },
    /// Shutdown the actor
    Shutdown,
}

/// Send-safe bridge for Tauri commands
#[derive(Clone)]
pub struct CrawlerBridge {
    sender: mpsc::UnboundedSender<ActorMessage>,
}

impl CrawlerBridge {
    /// Create a new crawler bridge and spawn the actor thread
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        // Spawn the actor in a dedicated thread (not tokio::spawn)
        thread::spawn(move || {
            let actor = CrawlerActor::new(receiver);
            actor.run();
        });

        Self { sender }
    }

    /// Start a new crawl session (async, Send-safe)
    pub async fn start_crawl(&self, request: CrawlRequest) -> Result<String, String> {
        let session_id = request.session_id.clone();
        let (response_tx, response_rx) = oneshot::channel();

        self.sender
            .send(ActorMessage::StartCrawl {
                session_id,
                request,
                response: response_tx,
            })
            .map_err(|_| "Actor is not running".to_string())?;

        response_rx
            .await
            .map_err(|_| "Actor response failed".to_string())?
    }

    /// Get status of a crawl session (async, Send-safe)
    pub async fn get_status(&self, session_id: String) -> Result<Option<CrawlStatus>, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.sender
            .send(ActorMessage::GetStatus {
                session_id,
                response: response_tx,
            })
            .map_err(|_| "Actor is not running".to_string())?;

        response_rx
            .await
            .map_err(|_| "Actor response failed".to_string())
    }

    /// Stop a crawl session (async, Send-safe)
    pub async fn stop_crawl(&self, session_id: String) -> Result<String, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.sender
            .send(ActorMessage::StopCrawl {
                session_id,
                response: response_tx,
            })
            .map_err(|_| "Actor is not running".to_string())?;

        response_rx
            .await
            .map_err(|_| "Actor response failed".to_string())?
    }
}

/// The actual crawler actor that owns non-Send types
struct CrawlerActor {
    receiver: mpsc::UnboundedReceiver<ActorMessage>,
    sessions: HashMap<String, CrawlStatus>,
    // Note: We'll store crawlers here when we support multiple concurrent sessions
}

impl CrawlerActor {
    fn new(receiver: mpsc::UnboundedReceiver<ActorMessage>) -> Self {
        Self {
            receiver,
            sessions: HashMap::new(),
        }
    }

    /// Main actor loop - runs in dedicated thread
    fn run(mut self) {
        println!("🎭 CrawlerActor started");

        // Create a simple tokio runtime for this thread
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        rt.block_on(async {
            while let Some(message) = self.receiver.recv().await {
                match message {
                    ActorMessage::StartCrawl {
                        session_id,
                        request,
                        response,
                    } => {
                        let result = self.handle_start_crawl(session_id, request).await;
                        let _ = response.send(result);
                    }
                    ActorMessage::GetStatus {
                        session_id,
                        response,
                    } => {
                        let status = self.sessions.get(&session_id).cloned();
                        let _ = response.send(status);
                    }
                    ActorMessage::StopCrawl {
                        session_id,
                        response,
                    } => {
                        let result = self.handle_stop_crawl(session_id);
                        let _ = response.send(result);
                    }
                    ActorMessage::Shutdown => {
                        println!("🎭 CrawlerActor shutting down");
                        break;
                    }
                }
            }
        });
    }

    /// Handle start crawl request
    async fn handle_start_crawl(
        &mut self,
        session_id: String,
        request: CrawlRequest,
    ) -> Result<String, String> {
        println!("🎭 Actor starting crawl for session: {}", session_id);

        // Initialize session status
        let mut status = CrawlStatus::new(&session_id);
        status.status = "running".to_string();
        status.current_url = Some(request.base_url.clone());
        self.sessions.insert(session_id.clone(), status);

        // Parse URL
        let url = match url::Url::parse(&request.base_url) {
            Ok(url) => url,
            Err(e) => {
                let error_msg = format!("Invalid URL: {}", e);
                self.set_session_error(&session_id, error_msg.clone());
                return Err(error_msg);
            }
        };

        // Create crawler config
        let config = WebCrawlerConfig {
            base_url: vec![request.base_url.clone()],
            max_crawl_depth: request.max_crawl_depth as usize,
            max_total_urls: request.max_total_urls as usize,
            target_words: request.target_words,
            enable_extension_crawling: request.enable_discovery_crawling,
            enable_keyword_filtering: request.enable_keyword_filtering,
            avoid_url_extensions: request.avoid_url_extensions,
            user_agent: "Tauri WebCrawler".to_string(),
            ..WebCrawlerConfig::default()
        };

        // Create and run crawler (this can use non-Send types safely in this thread)
        match WebCrawler::new(config, 5, 3) {
            Ok(crawler) => {
                match crawler.init_crawling(url).await {
                    Ok(content_opt) => {
                        println!("✅ Actor crawl completed for session: {}", session_id);

                        // Check if we have content and create results
                        let has_content = content_opt.is_some();
                        let crawl_results = if let Some(content) = content_opt {
                            vec![CrawlResultSummary {
                                url: request.base_url.clone(),
                                title: Some("Crawled Successfully".to_string()),
                                word_count: content.len(),
                                target_words_found: vec![], // TODO: Implement target word detection
                                language: Some("unknown".to_string()),
                                status_code: Some(200),
                            }]
                        } else {
                            vec![]
                        };

                        // Update session to completed
                        if let Some(status) = self.sessions.get_mut(&session_id) {
                            status.status = "completed".to_string();
                            status.total_urls_processed = 1;
                            status.successful_crawls = if has_content { 1 } else { 0 };
                            status.failed_crawls = if has_content { 0 } else { 1 };
                            status.results = crawl_results;
                        }

                        Ok(format!("Crawl completed for session: {}", session_id))
                    }
                    Err(e) => {
                        let error_msg = format!("Crawl failed: {}", e);
                        self.set_session_error(&session_id, error_msg.clone());
                        Err(error_msg)
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to create crawler: {}", e);
                self.set_session_error(&session_id, error_msg.clone());
                Err(error_msg)
            }
        }
    }

    /// Handle stop crawl request
    fn handle_stop_crawl(&mut self, session_id: String) -> Result<String, String> {
        if let Some(status) = self.sessions.get_mut(&session_id) {
            status.status = "stopped".to_string();
            Ok(format!("Crawl stopped for session: {}", session_id))
        } else {
            Err(format!("Session not found: {}", session_id))
        }
    }

    /// Set session to error state
    fn set_session_error(&mut self, session_id: &str, error: String) {
        if let Some(status) = self.sessions.get_mut(session_id) {
            status.status = "error".to_string();
            status.errors = vec![error];
        }
    }
}
