use anyhow::Result;
use reqwest::{Client, redirect::Policy};
use std::time::Duration;
use url::Url;

/// Test with "Tauri WebCrawler" user agent to see if that's the blocking factor
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing with 'Tauri WebCrawler' user agent");

    // Create client with WebCrawler config but Tauri user agent
    let client = Client::builder()
        .redirect(Policy::limited(10))
        .user_agent("Tauri WebCrawler") // This is what Tauri sets in config
        .timeout(Duration::from_secs(30))
        .build()?;

    let url = Url::parse("https://www.yahoo.com")?;

    println!("📡 Making request with 'Tauri WebCrawler' user agent...");

    // Test 1: Using the client's default user agent (Tauri WebCrawler)
    println!("\n=== Test 1: Client default user agent (Tauri WebCrawler) ===");
    let response1 = client.get(url.clone()).send().await?;
    let status1 = response1.status();
    let content1 = response1.text().await?;

    println!("Status: {}", status1);
    println!("Content size: {} bytes", content1.len());
    if content1.len() > 0 {
        println!("✅ SUCCESS with default client user agent");
    } else {
        println!("❌ FAILURE with default client user agent");
    }

    // Test 2: Override with browser user agent in header (like WebCrawler does)
    println!("\n=== Test 2: Override with browser user agent ===");
    let response2 = client
        .get(url.clone())
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .send()
        .await?;

    let status2 = response2.status();
    let content2 = response2.text().await?;

    println!("Status: {}", status2);
    println!("Content size: {} bytes", content2.len());
    if content2.len() > 0 {
        println!("✅ SUCCESS with browser user agent override");
    } else {
        println!("❌ FAILURE with browser user agent override");
    }

    Ok(())
}
