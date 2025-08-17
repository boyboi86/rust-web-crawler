use anyhow::{Error, Result};
use reqwest::{Client, redirect::Policy};
use std::time::Duration;
use url::Url;

/// Test with EXACT WebCrawler configuration to replicate Tauri behavior
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing with EXACT Tauri WebCrawler configuration");

    // Create client with EXACT same configuration as WebCrawler::new()
    let client = Client::builder()
        .redirect(Policy::limited(10)) // defaults::MAX_REDIRECTS
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(30)) // defaults::REQUEST_TIMEOUT_SECS
        .build()?;

    let url = Url::parse("https://www.yahoo.com")?;

    println!("📡 Making request with EXACT Tauri headers...");

    // Use EXACT same headers as WebCrawler::init_crawling()
    let response = client
        .get(url.clone())
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9") // Default fallback
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .send()
        .await?;

    let status = response.status();
    let content_length = response.content_length().unwrap_or(0);
    let content = response.text().await?;

    println!("✅ Response Details:");
    println!("   Status: {}", status);
    println!("   Content Length: {} bytes", content_length);
    println!("   Actual Content Size: {} bytes", content.len());

    if content.len() > 0 {
        println!("✅ SUCCESS: Content retrieved successfully!");
        println!("   First 200 chars: {}", &content[..content.len().min(200)]);

        // Check for blocking indicators
        if content.to_lowercase().contains("blocked")
            || content.to_lowercase().contains("forbidden")
            || content.to_lowercase().contains("access denied")
        {
            println!("⚠️  WARNING: Content may indicate blocking");
        }
    } else {
        println!("❌ FAILURE: Empty content received");
    }

    Ok(())
}
