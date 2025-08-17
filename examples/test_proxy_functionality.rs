use reqwest::{Client, Proxy};
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🧪 Testing Proxy Functionality Specifically");

    // Test 1: Direct connection (should work)
    println!("\n1️⃣ Testing direct connection...");
    match test_direct_connection().await {
        Ok(size) => println!("✅ Direct connection successful: {} bytes", size),
        Err(e) => println!("❌ Direct connection failed: {}", e),
    }

    // Test 2: Invalid proxy (should fail)
    println!("\n2️⃣ Testing with fake proxy (should fail)...");
    match test_with_fake_proxy().await {
        Ok(size) => println!("⚠️ Fake proxy worked unexpectedly: {} bytes", size),
        Err(e) => println!("✅ Fake proxy correctly failed: {}", e),
    }

    // Test 3: Public proxy test (might work)
    println!("\n3️⃣ Testing with potential public proxy...");
    match test_with_public_proxy().await {
        Ok(size) => println!("✅ Public proxy worked: {} bytes", size),
        Err(e) => println!("❌ Public proxy failed: {}", e),
    }

    // Test 4: Check what happens with our original test
    println!("\n4️⃣ Testing our original proxy setup...");
    match test_original_setup().await {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

async fn test_direct_connection() -> Result<usize, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let content = response.text().await?;
    println!("Direct IP response: {}", content.trim());
    Ok(content.len())
}

async fn test_with_fake_proxy() -> Result<usize, Box<dyn Error>> {
    let proxy = Proxy::http("http://fake-proxy.test:8080")?;

    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(5))
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let content = response.text().await?;
    Ok(content.len())
}

async fn test_with_public_proxy() -> Result<usize, Box<dyn Error>> {
    // Try a public HTTP proxy (note: these might not work or be reliable)
    let proxy = Proxy::http("http://proxy.server:3128")?; // Example proxy

    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let content = response.text().await?;
    println!("Proxy IP response: {}", content.trim());
    Ok(content.len())
}

async fn test_original_setup() -> Result<String, Box<dyn Error>> {
    // This simulates what our original test was doing
    let proxy_url = "http://us-proxy.test:8080";

    // Parse proxy URL to extract components
    let proxy_parts: Vec<&str> = proxy_url.trim_start_matches("http://").split(':').collect();
    if proxy_parts.len() != 2 {
        return Err("Invalid proxy URL format".into());
    }

    let proxy_host = proxy_parts[0];
    let proxy_port: u16 = proxy_parts[1].parse()?;

    println!(
        "Attempting to connect through proxy: {}:{}",
        proxy_host, proxy_port
    );

    // Create proxy configuration
    let proxy = Proxy::http(&format!("http://{}:{}", proxy_host, proxy_port))?;

    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(5))
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let content = response.text().await?;

    Ok(format!(
        "Unexpected success with fake proxy: {}",
        content.trim()
    ))
}
