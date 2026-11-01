use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};

pub struct BlockquoteHandler;

impl TagHandler for BlockquoteHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["blockquote"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let content = cx.children(element);
        let content = content.trim();

        if content.is_empty() {
            return String::new();
        }

        let quoted: String = content
            .lines()
            .map(|line| {
                if line.is_empty() {
                    ">".to_string()
                } else {
                    format!("> {}", line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("\n\n{}\n\n", quoted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render;

    fn convert_test(html: &str) -> String {
        render(&BlockquoteHandler, html)
    }

    #[test]
    fn test_simple_blockquote() {
        let result = convert_test("<blockquote>Quote</blockquote>");
        assert!(result.contains("> Quote"));
    }

    #[test]
    fn test_multiline_blockquote() {
        let result = convert_test("<blockquote>Line 1\nLine 2</blockquote>");
        assert!(result.contains("> Line 1"));
        assert!(result.contains("> Line 2"));
    }

    #[test]
    fn test_empty_blockquote() {
        let result = convert_test("<blockquote></blockquote>");
        assert!(result.is_empty());
    }
}
