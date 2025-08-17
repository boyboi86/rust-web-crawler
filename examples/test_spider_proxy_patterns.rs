/// Example demonstrating Spider-RS proxy patterns implementation
use anyhow::Result;
use rust_web_crawler::common::building_blocks::ReqwestClient;
use rust_web_crawler::network::proxy::{ProxyIgnore, ProxyRotationManager, RequestProxy};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Spider-RS Style Proxy Testing ===\n");

    // Step 1: Create some test proxies (since we don't have working source manager yet)
    println!("1. Setting up test proxies...");
    let test_proxies = vec![
        "185.199.229.156:7492",
        "185.199.228.220:7300",
        "185.199.231.45:8382",
        "188.74.210.207:6286",
        "188.74.183.10:8279",
    ];

    // Step 2: Convert to RequestProxy format with different ignore patterns
    let mut request_proxies = Vec::new();
    for (i, proxy) in test_proxies.into_iter().enumerate() {
        let ignore = match i % 3 {
            0 => ProxyIgnore::No,     // Use for both
            1 => ProxyIgnore::Chrome, // HTTP only
            2 => ProxyIgnore::Http,   // Chrome only
            _ => ProxyIgnore::No,
        };

        request_proxies.push(RequestProxy::new(format!("http://{}", proxy)).with_ignore(ignore));
    }

    println!(
        "   Created {} test proxies with different ignore patterns",
        request_proxies.len()
    );

    // Step 3: Set up rotation manager
    println!("2. Setting up proxy rotation manager...");
    let rotation_manager = ProxyRotationManager::new(request_proxies);

    // Step 4: Get HTTP proxies (Spider-RS style: all at once)
    let http_proxies = rotation_manager.get_http_proxies().await;
    println!("   HTTP proxies available: {}", http_proxies.len());

    // Step 5: Create client with all proxies (Spider-RS pattern)
    println!("3. Creating HTTP client with all proxies...");
    let client = match ReqwestClient::with_proxies(Duration::from_secs(10), http_proxies.clone()) {
        Ok(client) => {
            println!("   ✅ Client created with {} proxies", http_proxies.len());
            client
        }
        Err(e) => {
            println!("   ❌ Failed to create client: {}", e);
            println!("   📝 Falling back to direct connection...");
            ReqwestClient::with_timeout(Duration::from_secs(10))
        }
    };

    // Step 6: Test actual crawling functionality
    println!("4. Testing proxy functionality...");

    // Test 1: Check our IP
    println!("   Testing IP detection...");
    match test_ip_detection(&client).await {
        Ok(ip) => println!("   📍 Current IP: {}", ip),
        Err(e) => println!("   ❌ IP detection failed: {}", e),
    }

    // Test 2: Try crawling different sites
    let test_urls = vec![
        "https://httpbin.org/ip",
        "https://icanhazip.com",
        "https://api.ipify.org",
    ];

    for url in test_urls {
        match test_url(&client, url).await {
            Ok(response) => println!("   ✅ {}: {} bytes", url, response.len()),
            Err(e) => println!("   ❌ {}: {}", url, e),
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Step 7: Test proxy health monitoring
    println!("5. Testing proxy health monitoring...");
    let health_status = rotation_manager.get_health_status().await;
    for (addr, healthy, failures) in health_status {
        println!(
            "   📊 {}: {} (failures: {})",
            addr,
            if healthy { "✅ Healthy" } else { "❌ Failed" },
            failures
        );
    }

    // Step 8: Test round-robin rotation (alternative pattern)
    println!("6. Testing round-robin rotation...");
    for i in 0..5 {
        if let Some(proxy) = rotation_manager.get_next_proxy().await {
            println!("   🔄 Round {} proxy: {}", i + 1, proxy.addr);
        }
    }

    println!("\n=== Testing Complete ===");
    Ok(())
}

async fn test_ip_detection(client: &ReqwestClient) -> Result<String> {
    let response = client
        .client()
        .get("https://httpbin.org/ip")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await?;

    let text = response.text().await?;
    Ok(text.trim().to_string())
}

async fn test_url(client: &ReqwestClient, url: &str) -> Result<String> {
    let response = client
        .client()
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .send()
        .await?;

    Ok(response.text().await?)
}
