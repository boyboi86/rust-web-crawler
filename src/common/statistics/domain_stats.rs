use crate::common::primitives::{CountValue, LimitValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    domain_counts: HashMap<String, CountValue>,
    domain_limits: HashMap<String, LimitValue>,
}

impl DomainStats {
    pub fn new() -> Self {
        Self {
            domain_counts: HashMap::new(),
            domain_limits: HashMap::new(),
        }
    }
    pub fn increment_domain(&mut self, domain: &str) {
        self.domain_counts
            .entry(domain.to_string())
            .or_insert_with(CountValue::default)
            .increment();
    }
    pub fn get_domain_count(&self, domain: &str) -> u64 {
        self.domain_counts
            .get(domain)
            .map(|c| c.value())
            .unwrap_or(0)
    }
    pub fn set_domain_limit(&mut self, domain: String, limit: LimitValue) {
        self.domain_limits.insert(domain, limit);
    }
    pub fn is_domain_at_limit(&self, domain: &str) -> bool {
        let count = self.get_domain_count(domain);
        if let Some(limit) = self.domain_limits.get(domain) {
            limit.is_exceeded(count)
        } else {
            false
        }
    }
    pub fn get_top_domains(&self, count: usize) -> Vec<(String, u64)> {
        let mut domains: Vec<_> = self
            .domain_counts
            .iter()
            .map(|(domain, count)| (domain.clone(), count.value()))
            .collect();
        domains.sort_by(|a, b| b.1.cmp(&a.1));
        domains.truncate(count);
        domains
    }
    pub fn total_domains(&self) -> usize {
        self.domain_counts.len()
    }
    pub fn total_requests(&self) -> u64 {
        self.domain_counts.values().map(|c| c.value()).sum()
    }
    pub fn reset(&mut self) {
        self.domain_counts.clear();
    }
}

impl Default for DomainStats {
    fn default() -> Self {
        Self::new()
    }
}
