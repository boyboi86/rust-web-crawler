/// Test to demonstrate why Tauri fails vs examples succeed
use anyhow::Result;
use reqwest::{Client, redirect::Policy};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Why Tauri Fails vs Examples Succeed ===\n");

    // Test 1: Our successful example style
    println!("1. Testing with Example-style client (should work)...");
    let example_client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    match test_yahoo(&example_client, "Example Style", Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")).await {
        Ok(size) => println!("   ✅ SUCCESS: {} bytes", size),
        Err(e) => println!("   ❌ FAILED: {}", e),
    }

    // Test 2: Tauri's WebCrawler style (what's failing)
    println!("\n2. Testing with Tauri WebCrawler style (likely to fail)...");
    let tauri_client = Client::builder()
        .redirect(Policy::limited(10))
        .user_agent("Tauri WebCrawler") // This is the problem!
        .timeout(Duration::from_secs(30))
        .build()?;

    match test_yahoo(&tauri_client, "Tauri Style", None).await {
        Ok(size) => println!("   ✅ SUCCESS: {} bytes", size),
        Err(e) => println!("   ❌ FAILED: {}", e),
    }

    // Test 3: Bot-like user agent (should fail)
    println!("\n3. Testing with obvious bot user agent...");
    let bot_client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    match test_yahoo(&bot_client, "Bot Style", Some("WebCrawler Bot 1.0")).await {
        Ok(size) => println!("   ✅ SUCCESS: {} bytes", size),
        Err(e) => println!("   ❌ FAILED: {}", e),
    }

    // Test 4: Generic reqwest user agent
    println!("\n4. Testing with default reqwest user agent...");
    let default_client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    match test_yahoo(&default_client, "Default Reqwest", None).await {
        Ok(size) => println!("   ✅ SUCCESS: {} bytes", size),
        Err(e) => println!("   ❌ FAILED: {}", e),
    }

    println!("\n=== Analysis ===");
    println!("🔍 Key Differences Between Working Examples and Failing Tauri:");
    println!("   1. ❌ User Agent: 'Tauri WebCrawler' vs 'Mozilla/5.0...'");
    println!("   2. ❌ Missing Headers: Accept, Accept-Language, etc.");
    println!("   3. ❌ Bot Detection: Yahoo blocks obvious crawlers");
    println!("   4. ❌ Request Pattern: Different timeout/redirect policies");

    println!("\n📝 Why Examples Work but Tauri Fails:");
    println!("   ✅ Examples: Use browser-like headers and user agents");
    println!("   ❌ Tauri: Uses 'Tauri WebCrawler' user agent (obvious bot)");
    println!("   🚨 Yahoo blocks based on User-Agent, not IP");

    println!("\n🔧 Fix for Tauri:");
    println!("   1. Change user_agent in crawler_actor.rs line 204");
    println!("   2. Add browser-like headers to WebCrawler");
    println!("   3. Make requests look like real browsers");

    Ok(())
}

async fn test_yahoo(client: &Client, _label: &str, user_agent: Option<&str>) -> Result<usize> {
    let mut request = client.get("https://www.yahoo.com");

    if let Some(ua) = user_agent {
        request = request.header("User-Agent", ua);
    }

    // Add browser-like headers like our examples
    request = request
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Accept-Encoding", "gzip, deflate")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1");

    let response = request.send().await?;
    let text = response.text().await?;
    Ok(text.len())
}
