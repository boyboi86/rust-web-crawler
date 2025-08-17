use reqwest::{Client, Proxy};
use rust_web_crawler::network::proxy::source_manager::{FreeProxyListSource, ProxySource};
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🧪 Testing Real Proxy Crawling with Our System");

    // Get our baseline IP
    println!("\n1️⃣ Getting baseline IP...");
    let real_ip = get_real_ip().await?;
    println!("✅ Our real IP: {}", real_ip);

    // Fetch proxies using our system
    println!("\n2️⃣ Fetching proxies using FreeProxyListSource...");
    let source = FreeProxyListSource::new();

    match source.fetch_proxies().await {
        Ok(proxies) => {
            println!("✅ Fetched {} proxies", proxies.len());

            if !proxies.is_empty() {
                // Test first few proxies for actual crawling
                println!("\n3️⃣ Testing proxies for actual crawling...");
                test_proxies_for_crawling(&proxies, &real_ip).await?;
            }
        }
        Err(e) => {
            println!("❌ Failed to fetch proxies: {:?}", e);
        }
    }

    // Test with our geo selector
    println!("\n4️⃣ Testing integrated geo proxy selection...");
    test_geo_proxy_integration(&real_ip).await?;

    println!("\n🎉 Comprehensive proxy test completed!");
    Ok(())
}

async fn get_real_ip() -> Result<String, Box<dyn Error>> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let text = response.text().await?;

    if let Some(start) = text.find("\"origin\": \"") {
        let start = start + 11;
        if let Some(end) = text[start..].find("\"") {
            return Ok(text[start..start + end].to_string());
        }
    }

    Err("Could not parse IP".into())
}

async fn test_proxies_for_crawling(
    proxies: &[rust_web_crawler::network::proxy::source_manager::ProxyInfo],
    real_ip: &str,
) -> Result<(), Box<dyn Error>> {
    let mut working_proxies = 0;
    let mut tested_count = 0;

    // Test first 5 proxies
    for proxy in proxies.iter().take(5) {
        tested_count += 1;
        println!("   Testing proxy {}: {}", tested_count, proxy.url);

        match test_proxy_crawling(&proxy.url, real_ip).await {
            Ok((new_ip, content_size)) => {
                if new_ip != real_ip {
                    println!(
                        "   ✅ SUCCESS! IP changed {} → {}, crawled {} bytes",
                        real_ip, new_ip, content_size
                    );
                    working_proxies += 1;

                    // Test crawling a real website through this proxy
                    println!("      Testing real website crawl...");
                    match crawl_website_through_proxy(&proxy.url, "https://httpbin.org/html").await
                    {
                        Ok(size) => {
                            println!("      ✅ Successfully crawled website: {} bytes", size);
                        }
                        Err(e) => {
                            println!("      ❌ Website crawl failed: {}", e);
                        }
                    }
                } else {
                    println!("   ⚠️ Proxy didn't change IP (direct fallback)");
                }
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
            }
        }

        // Small delay between tests
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!(
        "\n📊 Results: {}/{} proxies worked successfully",
        working_proxies, tested_count
    );

    if working_proxies > 0 {
        println!("✅ Proxy system is functional for crawling!");
    } else {
        println!("⚠️ No working proxies found, but this is common with free proxy sources");
    }

    Ok(())
}

async fn test_proxy_crawling(
    proxy_url: &str,
    expected_different_from: &str,
) -> Result<(String, usize), Box<dyn Error>> {
    // Extract proxy components
    let proxy_addr = proxy_url.trim_start_matches("http://");
    let proxy = Proxy::http(&format!("http://{}", proxy_addr))?;

    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(8))
        .build()?;

    let response = client.get("https://httpbin.org/ip").send().await?;
    let text = response.text().await?;

    // Extract IP
    if let Some(start) = text.find("\"origin\": \"") {
        let start = start + 11;
        if let Some(end) = text[start..].find("\"") {
            let proxy_ip = text[start..start + end].to_string();
            return Ok((proxy_ip, text.len()));
        }
    }

    Err("Could not parse proxy response".into())
}

async fn crawl_website_through_proxy(
    proxy_url: &str,
    target_url: &str,
) -> Result<usize, Box<dyn Error>> {
    let proxy_addr = proxy_url.trim_start_matches("http://");
    let proxy = Proxy::http(&format!("http://{}", proxy_addr))?;

    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let response = client.get(target_url).send().await?;
    let content = response.text().await?;

    Ok(content.len())
}

async fn test_geo_proxy_integration(real_ip: &str) -> Result<(), Box<dyn Error>> {
    use rust_web_crawler::core::types::Region;
    use rust_web_crawler::network::proxy::GeoProxySelector;
    use url::Url;

    println!("   Setting up GeoProxySelector with working proxies...");
    let mut selector = GeoProxySelector::new();

    // Fetch some real proxies and assign them to regions
    let source = FreeProxyListSource::new();
    match source.fetch_proxies().await {
        Ok(proxies) => {
            if proxies.len() >= 3 {
                // Assign first few proxies to different regions
                selector.add_proxy_to_region(Region::NorthAmerica, proxies[0].url.clone());
                selector.add_proxy_to_region(Region::Europe, proxies[1].url.clone());
                selector.add_proxy_to_region(Region::AsiaPacific, proxies[2].url.clone());

                println!("   ✅ Configured proxies for all regions");

                // Test URL routing
                let test_cases = vec![
                    ("https://finance.yahoo.com", "North America"),
                    ("https://bbc.co.uk", "Europe"),
                    ("https://nikkei.com", "Asia Pacific"),
                ];

                for (url_str, expected_region) in test_cases {
                    let url = Url::parse(url_str)?;
                    if let Some(selected_proxy) = selector.select_proxy_for_url(&url) {
                        println!(
                            "   📍 {} → {} ({})",
                            url_str, selected_proxy, expected_region
                        );

                        // Optionally test crawling through the selected proxy
                        // (Skip for now to avoid overloading free proxies)
                    } else {
                        println!("   ❌ {} → No proxy selected", url_str);
                    }
                }
            } else {
                println!("   ⚠️ Not enough proxies to test geo routing");
            }
        }
        Err(e) => {
            println!("   ❌ Could not fetch proxies for geo test: {:?}", e);
        }
    }

    Ok(())
}
