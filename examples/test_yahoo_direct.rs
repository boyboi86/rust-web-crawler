/// Test direct crawling of Yahoo.com to check blocking behavior
use anyhow::Result;
use rust_web_crawler::common::building_blocks::ReqwestClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Testing Yahoo.com Direct Access ===\n");

    let client = ReqwestClient::with_timeout(Duration::from_secs(10));

    // Test Yahoo.com
    println!("1. Testing direct access to Yahoo.com...");
    match test_website(&client, "https://www.yahoo.com").await {
        Ok(content) => {
            println!("   ✅ Success! Received {} bytes", content.len());
            println!("   📄 Content preview (first 200 chars):");
            println!("   {}", &content[..content.len().min(200)]);

            // Check if it's a normal page or blocked
            if content.contains("<!DOCTYPE html") || content.contains("<html") {
                println!("   📝 Appears to be a normal HTML page - not blocked!");
            } else {
                println!("   ⚠️  Response doesn't look like HTML - might be blocked");
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {}", e);
        }
    }

    // Test other sites for comparison
    let test_sites = vec![
        ("https://httpbin.org/user-agent", "HTTPBin User-Agent"),
        ("https://example.com", "Example.com"),
        ("https://www.google.com", "Google"),
    ];

    println!("\n2. Testing other sites for comparison...");
    for (url, name) in test_sites {
        match test_website(&client, url).await {
            Ok(content) => {
                println!("   ✅ {}: {} bytes", name, content.len());
            }
            Err(e) => {
                println!("   ❌ {}: {}", name, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Test with different user agents
    println!("\n3. Testing Yahoo.com with different user agents...");
    let user_agents = vec![
        ("Default", None),
        (
            "Chrome",
            Some(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            ),
        ),
        (
            "Firefox",
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0"),
        ),
        (
            "Mobile",
            Some(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1",
            ),
        ),
    ];

    for (name, user_agent) in user_agents {
        match test_with_user_agent(&client, "https://www.yahoo.com", user_agent).await {
            Ok(content) => {
                println!("   ✅ {}: {} bytes", name, content.len());
            }
            Err(e) => {
                println!("   ❌ {}: {}", name, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    println!("\n=== Analysis ===");
    println!("If Yahoo.com works directly, then the issue with proxies is:");
    println!("1. ❌ Free proxies are dead/unreliable");
    println!("2. ✅ Our implementation is correct");
    println!("3. ✅ Spider-RS patterns work properly");
    println!("4. 📝 For production, use paid proxy services");

    Ok(())
}

async fn test_website(client: &ReqwestClient, url: &str) -> Result<String> {
    let response = client.client()
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Accept-Encoding", "gzip, deflate")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    Ok(response.text().await?)
}

async fn test_with_user_agent(
    client: &ReqwestClient,
    url: &str,
    user_agent: Option<&str>,
) -> Result<String> {
    let mut request = client.client().get(url);

    if let Some(ua) = user_agent {
        request = request.header("User-Agent", ua);
    }

    let response = request
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Accept-Encoding", "gzip, deflate")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    Ok(response.text().await?)
}
