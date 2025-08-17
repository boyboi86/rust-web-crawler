use reqwest::{Client, Proxy};
use rust_web_crawler::core::types::Region;
use rust_web_crawler::network::proxy::{GeoProxySelector, ProxyRegionsConfig};
use std::error::Error;
use std::time::Duration;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🧪 Testing Geo Proxy Selector Configuration with Real Crawling");

    // Test 1: Loading from config file
    println!("\n1️⃣ Testing config file loading...");
    match GeoProxySelector::with_config("src/config.toml") {
        Ok(mut selector) => {
            println!("✅ Config loaded successfully");

            // Add some test proxies (these are fake test proxies)
            selector.add_proxy_to_region(
                Region::NorthAmerica,
                "http://us-proxy.test:8080".to_string(),
            );
            selector.add_proxy_to_region(Region::Europe, "http://eu-proxy.test:8080".to_string());
            selector
                .add_proxy_to_region(Region::AsiaPacific, "http://sg-proxy.test:8080".to_string());

            // Test domain detection
            let test_urls = vec![
                "https://finance.yahoo.com",
                "https://bbc.co.uk",
                "https://nikkei.com",
                "https://unknown-site.com",
            ];

            for url_str in test_urls {
                let url = Url::parse(url_str)?;
                if let Some(proxy) = selector.select_proxy_for_url(&url) {
                    println!("📍 {} → {}", url_str, proxy);

                    // Try to crawl through the proxy
                    println!("🔄 Testing crawl through proxy...");
                    match test_crawl_with_proxy(url_str, &proxy).await {
                        Ok(content_length) => {
                            println!("✅ Successfully crawled {} bytes", content_length);
                        }
                        Err(e) => {
                            println!("❌ Failed to crawl through proxy: {}", e);
                            // Fall back to direct connection
                            println!("🔄 Trying direct connection...");
                            match test_crawl_direct(url_str).await {
                                Ok(content_length) => {
                                    println!(
                                        "✅ Direct crawl successful: {} bytes",
                                        content_length
                                    );
                                }
                                Err(e) => {
                                    println!("❌ Direct crawl also failed: {}", e);
                                }
                            }
                        }
                    }
                } else {
                    println!("❌ {} → No proxy found", url_str);
                }
                println!(""); // Separator
            }
        }
        Err(e) => {
            println!("⚠️ Config file not found, testing fallback: {}", e);

            // Test 2: Fallback to default mappings
            println!("\n2️⃣ Testing default mappings...");
            let mut selector = GeoProxySelector::new();
            selector.init_default_domain_mappings();

            selector.add_proxy_to_region(
                Region::NorthAmerica,
                "http://us-proxy.test:8080".to_string(),
            );

            let yahoo_url = Url::parse("https://yahoo.com")?;
            if let Some(proxy) = selector.select_proxy_for_url(&yahoo_url) {
                println!("✅ yahoo.com → {}", proxy);

                // Test crawling
                println!("🔄 Testing crawl...");
                match test_crawl_direct("https://yahoo.com").await {
                    Ok(content_length) => {
                        println!("✅ Successfully crawled {} bytes", content_length);
                    }
                    Err(e) => {
                        println!("❌ Failed to crawl: {}", e);
                    }
                }
            }
        }
    }

    println!("\n🎉 Tests completed!");
    Ok(())
}

async fn test_crawl_with_proxy(url: &str, proxy_url: &str) -> Result<usize, Box<dyn Error>> {
    // Parse proxy URL to extract components
    let proxy_parts: Vec<&str> = proxy_url.trim_start_matches("http://").split(':').collect();
    if proxy_parts.len() != 2 {
        return Err("Invalid proxy URL format".into());
    }

    let proxy_host = proxy_parts[0];
    let proxy_port: u16 = proxy_parts[1].parse()?;

    // Create proxy configuration
    let proxy = Proxy::http(&format!("http://{}:{}", proxy_host, proxy_port))?;

    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client.get(url).send().await?;
    let content = response.text().await?;
    Ok(content.len())
}

async fn test_crawl_direct(url: &str) -> Result<usize, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let response = client.get(url).send().await?;
    let content = response.text().await?;
    Ok(content.len())
}
