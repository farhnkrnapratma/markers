use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};
use crate::text::whitespace::collapse_whitespace;

pub struct DetailsHandler;

impl TagHandler for DetailsHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["details"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let mut summary = String::new();
        let mut content = String::new();

        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                if el.value().name() == "summary" {
                    let s = cx.children(el);
                    summary = collapse_whitespace(s.trim());
                } else {
                    content.push_str(&cx.children(el));
                }
            } else if let Some(text) = child.value().as_text() {
                content.push_str(text);
            }
        }

        let content = content.trim();

        if summary.is_empty() && content.is_empty() {
            return String::new();
        }

        let mut result = String::from("\n\n");
        if !summary.is_empty() {
            result.push_str(&format!("> **{}**\n>\n", summary));
        }
        for line in content.lines() {
            if line.is_empty() {
                result.push_str(">\n");
            } else {
                result.push_str(&format!("> {}\n", line));
            }
        }
        result.push('\n');
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render;

    fn convert_test(html: &str) -> String {
        render(&DetailsHandler, html)
    }

    #[test]
    fn test_details_with_summary() {
        let result = convert_test(
            r#"<details>
                <summary>Click to expand</summary>
                <p>Hidden content</p>
            </details>"#,
        );
        assert!(result.contains("> **Click to expand**"));
        assert!(result.contains("> Hidden content"));
    }

    #[test]
    fn test_details_without_summary() {
        let result = convert_test("<details><p>Just content</p></details>");
        assert!(result.contains("> Just content"));
        assert!(!result.contains("**"));
    }

    #[test]
    fn test_empty_details() {
        let result = convert_test("<details></details>");
        assert!(result.is_empty());
    }
}
