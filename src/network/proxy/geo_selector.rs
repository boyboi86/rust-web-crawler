// Enhanced proxy selection logic for geographical matching
use crate::core::types::Region;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use url::Url;

/// Configuration structure for loading domain mappings from TOML
#[derive(Debug, Deserialize, Serialize)]
pub struct ProxyRegionsConfig {
    pub proxy_regions: HashMap<String, Vec<String>>,
}

/// Intelligent proxy selector that matches proxies to target domains
pub struct GeoProxySelector {
    region_map: HashMap<String, Region>,
    proxy_pools: HashMap<Region, Vec<String>>,
    domain_to_region: HashMap<String, Region>,
    config_path: Option<String>,
}

impl GeoProxySelector {
    pub fn new() -> Self {
        Self {
            region_map: HashMap::new(),
            proxy_pools: HashMap::new(),
            domain_to_region: HashMap::new(),
            config_path: None,
        }
    }

    /// Create new selector with configuration file path
    pub fn with_config<P: AsRef<Path>>(config_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut selector = Self::new();
        selector.config_path = Some(config_path.as_ref().to_string_lossy().to_string());
        selector.load_config_from_file()?;
        Ok(selector)
    }

    /// Load domain mappings from TOML configuration file
    pub fn load_config_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = self.config_path.as_ref().ok_or("No config path set")?;

        let config_content = std::fs::read_to_string(config_path)?;
        let config: ProxyRegionsConfig = toml::from_str(&config_content)?;

        // Clear existing mappings
        self.domain_to_region.clear();

        // Load mappings from config
        for (region_str, domains) in config.proxy_regions {
            if let Ok(region) = region_str.parse::<Region>() {
                for domain in domains {
                    self.domain_to_region.insert(domain, region.clone());
                }
            }
        }

        Ok(())
    }

    /// Initialize known domain-to-region mappings (fallback for when no config file)
    pub fn init_default_domain_mappings(&mut self) {
        // Minimal fallback mappings - prefer loading from config file
        // Major US sites
        self.domain_to_region
            .insert("yahoo.com".to_string(), Region::NorthAmerica);
        self.domain_to_region
            .insert("google.com".to_string(), Region::NorthAmerica);

        // European sites
        self.domain_to_region
            .insert("bbc.co.uk".to_string(), Region::Europe);

        // Asia-Pacific sites
        self.domain_to_region
            .insert("sg.yahoo.com".to_string(), Region::AsiaPacific);

        // Chinese sites
        self.domain_to_region
            .insert("baidu.com".to_string(), Region::China);
    }

    /// Smart proxy selection based on target URL
    pub fn select_proxy_for_url(&self, url: &Url) -> Option<String> {
        let domain = url.host_str()?;

        // 1. Try exact domain match
        if let Some(region) = self.domain_to_region.get(domain) {
            return self.get_random_proxy_for_region(region);
        }

        // 2. Try subdomain detection (e.g., news.yahoo.com -> yahoo.com)
        if let Some(parent_domain) = self.extract_parent_domain(domain) {
            if let Some(region) = self.domain_to_region.get(&parent_domain) {
                return self.get_random_proxy_for_region(region);
            }
        }

        // 3. Try country code detection (e.g., .co.uk -> Europe, .jp -> Asia)
        if let Some(region) = self.detect_region_from_tld(domain) {
            return self.get_random_proxy_for_region(&region);
        }

        // 4. Try geographic subdomain detection (e.g., sg.yahoo.com -> Asia)
        if let Some(region) = self.detect_region_from_subdomain(domain) {
            return self.get_random_proxy_for_region(&region);
        }

        // 5. Fallback to global proxy pool
        self.get_random_proxy_for_region(&Region::Global)
    }

    /// Extract parent domain (news.yahoo.com -> yahoo.com)
    fn extract_parent_domain(&self, domain: &str) -> Option<String> {
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() >= 3 {
            Some(parts[parts.len() - 2..].join("."))
        } else {
            None
        }
    }

    /// Detect region from Top-Level Domain
    fn detect_region_from_tld(&self, domain: &str) -> Option<Region> {
        if domain.ends_with(".com") || domain.ends_with(".org") || domain.ends_with(".net") {
            Some(Region::NorthAmerica)
        } else if domain.ends_with(".eu")
            || domain.ends_with(".de")
            || domain.ends_with(".fr")
            || domain.ends_with(".uk")
            || domain.ends_with(".co.uk")
        {
            Some(Region::Europe)
        } else if domain.ends_with(".jp")
            || domain.ends_with(".kr")
            || domain.ends_with(".sg")
            || domain.ends_with(".au")
            || domain.ends_with(".com.au")
        {
            Some(Region::AsiaPacific)
        } else if domain.ends_with(".cn") || domain.ends_with(".com.cn") {
            Some(Region::China)
        } else {
            None
        }
    }

    /// Detect region from subdomain (sg.yahoo.com, uk.yahoo.com)
    fn detect_region_from_subdomain(&self, domain: &str) -> Option<Region> {
        let parts: Vec<&str> = domain.split('.').collect();
        if let Some(subdomain) = parts.first() {
            match *subdomain {
                "us" | "www" => Some(Region::NorthAmerica),
                "uk" | "de" | "fr" | "eu" | "es" | "it" => Some(Region::Europe),
                "sg" | "jp" | "kr" | "au" | "hk" | "tw" => Some(Region::AsiaPacific),
                "cn" => Some(Region::China),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get random proxy from specific region pool
    fn get_random_proxy_for_region(&self, region: &Region) -> Option<String> {
        use rand::Rng;

        if let Some(proxies) = self.proxy_pools.get(region) {
            if !proxies.is_empty() {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..proxies.len());
                return Some(proxies[index].clone());
            }
        }

        // Fallback to global pool if region-specific pool is empty
        if region != &Region::Global {
            return self.get_random_proxy_for_region(&Region::Global);
        }

        None
    }

    /// Add proxy to specific region
    pub fn add_proxy_to_region(&mut self, region: Region, proxy: String) {
        self.proxy_pools
            .entry(region)
            .or_insert_with(Vec::new)
            .push(proxy);
    }

    /// Load proxies from configuration
    pub fn load_proxy_configuration(&mut self, config: &ProxyConfig) {
        for (region_str, proxies) in &config.regional_proxies {
            if let Ok(region) = region_str.parse::<Region>() {
                self.proxy_pools.insert(region, proxies.clone());
            }
        }
    }
}

#[derive(Debug)]
pub struct ProxyConfig {
    pub regional_proxies: HashMap<String, Vec<String>>,
    pub auto_detect_region: bool,
    pub fallback_to_global: bool,
}

// Usage example:
impl GeoProxySelector {
    pub fn example_usage() {
        // Method 1: Load from config file (recommended)
        let mut selector =
            GeoProxySelector::with_config("src/config.toml").expect("Failed to load config");

        // Method 2: Use default mappings (fallback)
        let mut fallback_selector = GeoProxySelector::new();
        fallback_selector.init_default_domain_mappings();

        // Add proxies by region
        selector.add_proxy_to_region(
            Region::NorthAmerica,
            "http://us-proxy1.com:8080".to_string(),
        );
        selector.add_proxy_to_region(Region::Europe, "http://de-proxy1.com:8080".to_string());
        selector.add_proxy_to_region(Region::AsiaPacific, "http://sg-proxy1.com:8080".to_string());

        // Smart selection
        let yahoo_url = Url::parse("https://finance.yahoo.com").unwrap();
        let selected_proxy = selector.select_proxy_for_url(&yahoo_url);
        // Will select a North American proxy for better success

        let sg_yahoo_url = Url::parse("https://sg.yahoo.com").unwrap();
        let selected_proxy = selector.select_proxy_for_url(&sg_yahoo_url);
        // Will select an Asia-Pacific proxy for better regional matching
    }
}
