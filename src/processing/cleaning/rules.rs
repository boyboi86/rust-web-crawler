/// Text cleaning rules and engine
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::error::CrawlError;

/// Type of cleaning rule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    /// Remove specific text patterns
    Remove,
    /// Replace text patterns
    Replace,
    /// Keep only specific patterns
    KeepOnly,
    /// Transform text (uppercase, lowercase, etc.)
    Transform,
}

/// Individual cleaning rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningRule {
    /// Name/description of the rule
    pub name: String,
    /// Type of rule
    pub rule_type: RuleType,
    /// Pattern to match (regex)
    pub pattern: String,
    /// Replacement text (for Replace rules)
    pub replacement: Option<String>,
    /// Whether rule is enabled
    pub enabled: bool,
    /// Priority (higher = applied first)
    pub priority: u8,
}

impl CleaningRule {
    /// Create a new removal rule
    pub fn remove(name: &str, pattern: &str) -> Self {
        Self {
            name: name.to_string(),
            rule_type: RuleType::Remove,
            pattern: pattern.to_string(),
            replacement: None,
            enabled: true,
            priority: 50,
        }
    }

    /// Create a new replacement rule
    pub fn replace(name: &str, pattern: &str, replacement: &str) -> Self {
        Self {
            name: name.to_string(),
            rule_type: RuleType::Replace,
            pattern: pattern.to_string(),
            replacement: Some(replacement.to_string()),
            enabled: true,
            priority: 50,
        }
    }

    /// Validate the rule
    pub fn validate(&self) -> Result<(), CrawlError> {
        // Check if pattern is valid regex
        if let Err(e) = Regex::new(&self.pattern) {
            return Err(CrawlError::CleaningRuleError(format!(
                "Invalid regex pattern in rule '{}': {}",
                self.name, e
            )));
        }

        // Check if replacement is provided for replace rules
        if self.rule_type == RuleType::Replace && self.replacement.is_none() {
            return Err(CrawlError::CleaningRuleError(format!(
                "Replace rule '{}' must have a replacement value",
                self.name
            )));
        }

        Ok(())
    }
}

/// Cleaning engine that applies rules
pub struct CleaningEngine {
    rules: Vec<(CleaningRule, Regex)>,
}

impl CleaningEngine {
    /// Create a new cleaning engine
    pub fn new(mut rules: Vec<CleaningRule>) -> Result<Self, CrawlError> {
        // Validate and compile rules
        let mut compiled_rules = Vec::new();

        // Sort rules by priority (highest first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        for rule in rules {
            if !rule.enabled {
                continue;
            }

            rule.validate()?;

            let regex = Regex::new(&rule.pattern).map_err(|e| {
                CrawlError::CleaningRuleError(format!(
                    "Failed to compile rule '{}': {}",
                    rule.name, e
                ))
            })?;

            compiled_rules.push((rule, regex));
        }

        Ok(Self {
            rules: compiled_rules,
        })
    }

    /// Apply all rules to text
    pub fn apply_rules(&self, text: &str) -> String {
        let mut result = text.to_string();

        for (rule, regex) in &self.rules {
            result = match rule.rule_type {
                RuleType::Remove => regex.replace_all(&result, "").to_string(),
                RuleType::Replace => {
                    if let Some(ref replacement) = rule.replacement {
                        regex.replace_all(&result, replacement).to_string()
                    } else {
                        result
                    }
                }
                RuleType::KeepOnly => {
                    let matches: Vec<String> = regex
                        .find_iter(&result)
                        .map(|m| m.as_str().to_string())
                        .collect();
                    matches.join(" ")
                }
                RuleType::Transform => {
                    // Basic transformations
                    match rule.pattern.as_str() {
                        "lowercase" => result.to_lowercase(),
                        "uppercase" => result.to_uppercase(),
                        "trim" => result.trim().to_string(),
                        _ => result,
                    }
                }
            };
        }

        result
    }

    /// Get predefined rule sets
    pub fn default_rules() -> Vec<CleaningRule> {
        vec![
            CleaningRule::remove("remove_html_tags", r"<[^>]*>"),
            CleaningRule::remove("remove_extra_whitespace", r"\s+"),
            CleaningRule::remove("remove_urls", r"https?://[^\s]+"),
            CleaningRule::remove(
                "remove_email",
                r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
            ),
            CleaningRule::replace("normalize_quotes", r"[''`]", "'"),
            CleaningRule::replace("normalize_dashes", r"[–—]", "-"),
        ]
    }

    /// Get CJK removal rules
    pub fn cjk_removal_rules() -> Vec<CleaningRule> {
        vec![
            CleaningRule::remove("remove_chinese", r"[\u4e00-\u9fff]+"),
            CleaningRule::remove("remove_japanese_hiragana", r"[\u3040-\u309f]+"),
            CleaningRule::remove("remove_japanese_katakana", r"[\u30a0-\u30ff]+"),
            CleaningRule::remove("remove_korean", r"[\uac00-\ud7af]+"),
        ]
    }

    /// Get aggressive cleaning rules
    pub fn aggressive_rules() -> Vec<CleaningRule> {
        vec![
            CleaningRule::remove("remove_non_ascii", r"[^\x00-\x7F]+"),
            CleaningRule::remove("remove_numbers", r"\d+"),
            CleaningRule::remove("remove_special_chars", r"[^\w\s]"),
            CleaningRule::replace("normalize_whitespace", r"\s+", " "),
        ]
    }
}
