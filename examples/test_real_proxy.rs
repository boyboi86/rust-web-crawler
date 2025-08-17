use reqwest::{Client, Proxy};
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🧪 Testing REAL Proxy Functionality");

    // Test 1: Get our real IP first
    println!("\n1️⃣ Getting our real IP address...");
    let real_ip = get_real_ip().await?;
    println!("✅ Our real IP: {}", real_ip);

    // Test 2: Test with clearly invalid proxy (should fail)
    println!("\n2️⃣ Testing with invalid proxy (should fail)...");
    match test_with_invalid_proxy().await {
        Ok(response) => {
            println!("⚠️ WARNING: Invalid proxy somehow worked! Response: {}", response);
            println!("   This suggests proxy configuration is not being applied properly");
        }
        Err(e) => {
            println!("✅ Good: Invalid proxy correctly failed: {}", e);
        }
    }

    // Test 3: Let's get some actual working proxies and test
    println!("\n3️⃣ Testing with actual proxy sources...");
    match fetch_and_test_real_proxies().await {
        Ok(results) => {
            println!("✅ Found working proxies:");
            for (proxy, ip) in results {
                println!("   {} → IP: {}", proxy, ip);
            }
        }
        Err(e) => {
            println!("❌ Failed to fetch real proxies: {}", e);
        }
    }

    // Test 4: Test our ProxyProvider building blocks
    println!("\n4️⃣ Testing ProxyProvider building blocks...");
    test_proxy_provider().await?;

    Ok(())
}

async fn get_real_ip() -> Result<String, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let text = response.text().await?;
    
    // Extract just the IP from the JSON response
    if let Some(start) = text.find("\"origin\": \"") {
        let start = start + 11;
        if let Some(end) = text[start..].find("\"") {
            return Ok(text[start..start + end].to_string());
        }
    }
    
    Err("Could not parse IP from response".into())
}

async fn test_with_invalid_proxy() -> Result<String, Box<dyn Error>> {
    // Use a completely invalid proxy that should definitely fail
    let proxy = Proxy::http("http://192.0.2.1:8080")?; // RFC5737 test address
    
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(3)) // Short timeout
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let content = response.text().await?;
    Ok(content)
}

async fn fetch_and_test_real_proxies() -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut working_proxies = Vec::new();
    
    // Try to fetch from a real free proxy source
    println!("   Fetching from free proxy API...");
    
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    // Try proxyscrape
    match client.get("https://api.proxyscrape.com/v2/?request=get&protocol=http&timeout=10000&country=US&format=textplain").send().await {
        Ok(response) => {
            let proxy_list = response.text().await?;
            println!("   Downloaded proxy list, testing proxies...");
            
            // Test first few proxies from the list
            for (i, line) in proxy_list.lines().take(3).enumerate() {
                let line = line.trim();
                if line.is_empty() || !line.contains(':') {
                    continue;
                }
                
                println!("   Testing proxy {}: {}", i + 1, line);
                match test_single_proxy(line).await {
                    Ok(ip) => {
                        println!("   ✅ Proxy {} works! IP: {}", line, ip);
                        working_proxies.push((line.to_string(), ip));
                    }
                    Err(e) => {
                        println!("   ❌ Proxy {} failed: {}", line, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to fetch proxy list: {}", e);
        }
    }
    
    Ok(working_proxies)
}

async fn test_single_proxy(proxy_address: &str) -> Result<String, Box<dyn Error>> {
    let proxy_url = format!("http://{}", proxy_address);
    let proxy = Proxy::http(&proxy_url)?;
    
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(8))
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let text = response.text().await?;
    
    // Extract IP from response
    if let Some(start) = text.find("\"origin\": \"") {
        let start = start + 11;
        if let Some(end) = text[start..].find("\"") {
            return Ok(text[start..start + end].to_string());
        }
    }
    
    Err("Could not parse IP from proxy response".into())
}

async fn test_proxy_provider() -> Result<(), Box<dyn Error>> {
    use rust_web_crawler::network::proxy::{ProxyProvider, ProxyProviderConfig};
    
    println!("   Creating ProxyProvider with default config...");
    let config = ProxyProviderConfig::default();
    let provider = ProxyProvider::new(config);
    
    println!("   ✅ ProxyProvider created successfully");
    println!("   Config timeout: {} seconds", provider.config().timeout_seconds);
    println!("   Config max retries: {}", provider.config().max_retries);
    println!("   Free sources configured: {}", provider.config().free_sources.len());
    
    Ok(())
}
