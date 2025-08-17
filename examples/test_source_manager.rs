use rust_web_crawler::network::proxy::source_manager::{FreeProxyListSource, ProxySource};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🧪 Testing Our Proxy Source Manager Implementation");

    // Test 1: FreeProxyListSource
    println!("\n1️⃣ Testing FreeProxyListSource...");
    let source = FreeProxyListSource::new();

    match source.fetch_proxies().await {
        Ok(proxies) => {
            println!("✅ Successfully fetched {} proxies", proxies.len());

            if proxies.is_empty() {
                println!("⚠️ No proxies returned - this might indicate an API issue");
            } else {
                println!("📋 First 5 proxies:");
                for (i, proxy) in proxies.iter().take(5).enumerate() {
                    println!(
                        "   {}. {} (Region: {:?}, Success Rate: {:.1}%)",
                        i + 1,
                        proxy.url,
                        proxy.region,
                        proxy.success_rate * 100.0
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to fetch proxies: {:?}", e);

            // Let's try to understand what went wrong
            println!("\n🔍 Debugging the issue...");
            test_direct_api_call().await?;
        }
    }

    println!("\n🎉 Source manager test completed!");
    Ok(())
}

async fn test_direct_api_call() -> Result<(), Box<dyn Error>> {
    use reqwest::Client;
    use std::time::Duration;

    println!("   Making direct API call to verify endpoint...");

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let url = "https://www.proxy-list.download/api/v1/get?type=http";

    match client.get(url).send().await {
        Ok(response) => {
            println!("   ✅ API call successful, status: {}", response.status());

            let content = response.text().await?;
            let lines: Vec<&str> = content.lines().collect();

            println!("   📄 Response has {} lines", lines.len());

            if lines.len() > 0 {
                println!("   📋 First 3 lines:");
                for (i, line) in lines.iter().take(3).enumerate() {
                    println!("      {}. '{}'", i + 1, line);
                }
            }

            if content.trim().is_empty() {
                println!("   ⚠️ API returned empty response");
            }
        }
        Err(e) => {
            println!("   ❌ API call failed: {}", e);
        }
    }

    Ok(())
}
