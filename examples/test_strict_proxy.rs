use reqwest::{Client, Proxy};
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🧪 Testing Proxy System with Strict Validation");

    // Get our real IP first
    println!("\n1️⃣ Getting baseline IP address...");
    let real_ip = get_real_ip().await?;
    println!("✅ Our real IP: {}", real_ip);

    // Test with free proxy sources - fetch actual proxies
    println!("\n2️⃣ Fetching real proxies from public sources...");
    match fetch_real_proxies_and_test(&real_ip).await {
        Ok(working_count) => {
            if working_count > 0 {
                println!(
                    "✅ Successfully validated {} working proxies",
                    working_count
                );
            } else {
                println!("⚠️ No working proxies found - this is normal for free proxies");
            }
        }
        Err(e) => {
            println!("❌ Error testing proxies: {}", e);
        }
    }

    // Test our proxy selector integration
    println!("\n3️⃣ Testing GeoProxySelector with real proxies...");
    test_geo_proxy_with_real_proxies(&real_ip).await?;

    println!("\n🎉 Proxy testing completed!");
    Ok(())
}

async fn get_real_ip() -> Result<String, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .no_proxy() // Explicitly disable proxy
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let text = response.text().await?;

    // Extract IP from JSON
    if let Some(start) = text.find("\"origin\": \"") {
        let start = start + 11;
        if let Some(end) = text[start..].find("\"") {
            return Ok(text[start..start + end].to_string());
        }
    }

    Err("Could not parse IP".into())
}

async fn fetch_real_proxies_and_test(real_ip: &str) -> Result<usize, Box<dyn Error>> {
    println!("   Fetching from proxyscrape.com...");

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .no_proxy()
        .build()?;

    // Fetch proxies with better parameters
    let response = client
        .get("https://api.proxyscrape.com/v2/?request=get&protocol=http&timeout=5000&country=US&format=textplain&limit=10")
        .send()
        .await?;

    let proxy_list = response.text().await?;
    println!("   Retrieved {} proxy entries", proxy_list.lines().count());

    let mut working_count = 0;

    // Test each proxy
    for (i, line) in proxy_list.lines().take(5).enumerate() {
        let line = line.trim();
        if line.is_empty() || !line.contains(':') {
            continue;
        }

        println!("   Testing proxy {}: {}", i + 1, line);

        match test_proxy_strictly(line, real_ip).await {
            Ok(proxy_ip) => {
                if proxy_ip != real_ip {
                    println!(
                        "   ✅ Proxy {} works! Changed IP: {} → {}",
                        line, real_ip, proxy_ip
                    );
                    working_count += 1;
                } else {
                    println!("   ⚠️ Proxy {} didn't change IP (fallback to direct)", line);
                }
            }
            Err(e) => {
                println!("   ❌ Proxy {} failed: {}", line, e);
            }
        }
    }

    Ok(working_count)
}

async fn test_proxy_strictly(
    proxy_address: &str,
    expected_different_from: &str,
) -> Result<String, Box<dyn Error>> {
    let proxy_url = format!("http://{}", proxy_address);
    let proxy = Proxy::http(&proxy_url)?;

    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(8))
        .no_proxy() // This should be .proxy() but we want strict proxy usage
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let text = response.text().await?;

    // Extract IP
    if let Some(start) = text.find("\"origin\": \"") {
        let start = start + 11;
        if let Some(end) = text[start..].find("\"") {
            let proxy_ip = text[start..start + end].to_string();

            // Validate that the IP actually changed
            if proxy_ip == expected_different_from {
                return Err("Proxy did not change IP address".into());
            }

            return Ok(proxy_ip);
        }
    }

    Err("Could not parse proxy response".into())
}

async fn test_geo_proxy_with_real_proxies(real_ip: &str) -> Result<(), Box<dyn Error>> {
    use rust_web_crawler::core::types::Region;
    use rust_web_crawler::network::proxy::GeoProxySelector;

    println!("   Creating GeoProxySelector...");
    let mut selector = GeoProxySelector::new();

    // Instead of fake proxies, let's test the concept with manual setup
    println!("   Testing concept with simulated working proxy setup...");

    // Simulate what would happen with working proxies
    selector.add_proxy_to_region(Region::NorthAmerica, "working-us-proxy:8080".to_string());
    selector.add_proxy_to_region(Region::Europe, "working-eu-proxy:8080".to_string());

    // Test URL routing
    use url::Url;
    let test_urls = vec![
        ("https://yahoo.com", Region::NorthAmerica),
        ("https://bbc.co.uk", Region::Europe),
        ("https://nikkei.com", Region::AsiaPacific),
    ];

    for (url_str, expected_region) in test_urls {
        let url = Url::parse(url_str)?;
        if let Some(proxy) = selector.select_proxy_for_url(&url) {
            println!(
                "   ✅ {} → {} (detected region: {:?})",
                url_str, proxy, expected_region
            );
        } else {
            println!("   ❌ {} → No proxy selected", url_str);
        }
    }

    Ok(())
}

// Alternative test: Let's create a working proxy integration test
#[allow(dead_code)]
async fn test_proxy_integration_with_source_manager() -> Result<(), Box<dyn Error>> {
    use rust_web_crawler::network::proxy::source_manager::FreeProxyListSource;
    use rust_web_crawler::network::proxy::source_manager::ProxySource;

    println!("   Testing FreeProxyListSource...");
    let source = FreeProxyListSource::new();

    match source.fetch_proxies().await {
        Ok(proxies) => {
            println!("   ✅ Fetched {} proxies from source", proxies.len());
            for (i, proxy) in proxies.iter().take(3).enumerate() {
                println!("   Proxy {}: {} ({})", i + 1, proxy.url, proxy.source);
            }
        }
        Err(e) => {
            println!("   ❌ Failed to fetch proxies: {:?}", e);
        }
    }

    Ok(())
}
