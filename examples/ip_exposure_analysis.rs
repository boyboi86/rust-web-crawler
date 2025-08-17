/// Test to clearly show IP exposure when testing Yahoo.com
use anyhow::Result;
use rust_web_crawler::common::building_blocks::ReqwestClient;
use rust_web_crawler::network::proxy::{ProxyRotationManager, RequestProxy};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== IP Exposure Analysis for Yahoo.com Testing ===\n");

    // Test 1: Check what IP we're showing
    println!("1. Checking our current IP (direct connection)...");
    let direct_client = ReqwestClient::with_timeout(Duration::from_secs(10));

    match check_ip(&direct_client, "Direct Connection").await {
        Ok(ip) => println!("   🚨 EXPOSED IP: {}", ip),
        Err(e) => println!("   ❌ IP check failed: {}", e),
    }

    // Test 2: Test with dead proxies (what we actually did)
    println!("\n2. Testing with the same dead proxies we used before...");
    let dead_proxies = vec![
        "185.199.229.156:7492",
        "185.199.228.220:7300",
        "185.199.231.45:8382",
        "188.74.210.207:6286",
        "188.74.183.10:8279",
    ];

    let request_proxies: Vec<RequestProxy> = dead_proxies
        .into_iter()
        .map(|proxy| RequestProxy::new(format!("http://{}", proxy)))
        .collect();

    let rotation_manager = ProxyRotationManager::new(request_proxies);
    let http_proxies = rotation_manager.get_http_proxies().await;

    println!("   📋 Trying to use {} proxies...", http_proxies.len());

    match ReqwestClient::with_proxies(Duration::from_secs(5), http_proxies) {
        Ok(proxy_client) => {
            println!("   📡 Client created with proxies, testing IP...");
            match check_ip(&proxy_client, "Through Proxies").await {
                Ok(ip) => {
                    println!("   🔍 Result: {}", ip);
                    if ip.contains("116.86.40.184") {
                        println!("   🚨 WARNING: Still showing your real IP!");
                        println!(
                            "   📝 This means proxies failed and fell back to direct connection"
                        );
                    } else {
                        println!("   ✅ SUCCESS: Using proxy IP!");
                    }
                }
                Err(e) => {
                    println!("   ❌ Proxy IP check failed: {}", e);
                    println!("   📝 This confirms proxies are dead");
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to create proxy client: {}", e);
        }
    }

    // Test 3: What Yahoo.com actually saw
    println!("\n3. What Yahoo.com actually saw when we tested...");
    println!("   📍 When we successfully got 447KB from Yahoo.com:");
    println!("   🚨 Yahoo saw your real IP: 116.86.40.184");
    println!("   📝 This happened because:");
    println!("      1. We tried to use dead proxies");
    println!("      2. Proxies failed/timed out");
    println!("      3. Reqwest fell back to direct connection");
    println!("      4. Yahoo.com received request from your real IP");

    // Test 4: Security implications
    println!("\n4. Security implications...");
    println!("   ⚠️  IP Exposure: Yahoo.com can see your real IP");
    println!("   ⚠️  Tracking: They can associate requests with your IP");
    println!("   ⚠️  Rate Limiting: Future requests from your IP might be limited");
    println!("   ⚠️  Geographic Detection: They know your approximate location");

    // Test 5: How to fix this
    println!("\n5. How to properly hide your IP for Yahoo.com crawling...");
    println!("   ✅ Use paid proxy services (BrightData, ProxyMesh, etc.)");
    println!("   ✅ Test proxy connectivity before crawling");
    println!("   ✅ Implement proper error handling for proxy failures");
    println!("   ✅ Use VPN as additional layer");
    println!("   ✅ Rotate user agents and headers");

    println!("\n=== SUMMARY ===");
    println!("❌ NO, we did not successfully crawl Yahoo.com through proxies");
    println!("🚨 YES, your real IP (116.86.40.184) was exposed to Yahoo.com");
    println!("✅ YES, we successfully retrieved data (447KB) from Yahoo.com");
    println!("📝 BUT this was through direct connection, not proxies");

    Ok(())
}

async fn check_ip(client: &ReqwestClient, label: &str) -> Result<String> {
    let response = client
        .client()
        .get("https://httpbin.org/ip")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    let text = response.text().await?;
    Ok(format!("{}: {}", label, text.trim()))
}
