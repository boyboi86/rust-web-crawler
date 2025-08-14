/// HTML parsing and extraction utilities
use regex::Regex;

/// Extract title from HTML content
pub fn extract_title_from_html(content: &str) -> Option<String> {
    let re = Regex::new(r"<title[^>]*>([^<]+)</title>").ok()?;
    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Extract links from HTML content
pub fn extract_links_from_html(content: &str) -> Vec<String> {
    let re = Regex::new(r#"href\s*=\s*["']([^"']+)["']"#).unwrap_or_else(|_| {
        // Fallback to a simpler pattern if the complex one fails
        Regex::new(r#"href=["']([^"']+)["']"#).unwrap()
    });

    re.captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|link| {
            // Filter out non-HTTP links
            link.starts_with("http") || link.starts_with("//")
        })
        .collect()
}

/// Extract meta description from HTML content
pub fn extract_meta_description(content: &str) -> Option<String> {
    let re = Regex::new(
        r#"<meta[^>]*name\s*=\s*["']description["'][^>]*content\s*=\s*["']([^"']*)["']"#,
    )
    .ok()?;
    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Extract all text content from HTML (strip tags)
pub fn extract_text_content(html: &str) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let text = re.replace_all(html, " ");

    // Clean up multiple spaces and normalize whitespace
    let clean_re = Regex::new(r"\s+").unwrap();
    clean_re.replace_all(&text, " ").trim().to_string()
}

/// Check if HTML content appears to be a valid page
pub fn is_valid_html_content(content: &str) -> bool {
    // Check for basic HTML structure
    content.contains("<html") || content.contains("<body") || content.contains("<head")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = r#"<html><head><title>Test Page</title></head><body></body></html>"#;
        assert_eq!(extract_title_from_html(html), Some("Test Page".to_string()));
    }

    #[test]
    fn test_extract_links() {
        let html =
            r#"<a href="https://example.com">Link 1</a><a href="http://test.com">Link 2</a>"#;
        let links = extract_links_from_html(html);
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com".to_string()));
        assert!(links.contains(&"http://test.com".to_string()));
    }

    #[test]
    fn test_extract_text_content() {
        let html = r#"<html><body><h1>Hello</h1><p>World</p></body></html>"#;
        let text = extract_text_content(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("<h1>"));
    }
}
