use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};
use crate::options::HeadingStyle;
use crate::text::whitespace::collapse_whitespace;

pub struct HeadingHandler;

impl TagHandler for HeadingHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["h1", "h2", "h3", "h4", "h5", "h6"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let tag = element.value().name();
        let level: usize = tag[1..].parse().unwrap_or(1);

        let content = cx.children(element);
        let content = collapse_whitespace(content.trim());

        if content.is_empty() {
            return String::new();
        }

        match cx.options.heading_style {
            HeadingStyle::Atx => {
                format!("\n\n{} {}\n\n", "#".repeat(level), content)
            }
            HeadingStyle::Setext if level <= 2 => {
                let underline = if level == 1 { "=" } else { "-" };
                let len = content.chars().count();
                format!("\n\n{}\n{}\n\n", content, underline.repeat(len))
            }
            _ => format!("\n\n{} {}\n\n", "#".repeat(level), content),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render_with;
    use crate::options::ConversionOptions;

    fn convert_test(html: &str, options: &ConversionOptions) -> String {
        render_with(&HeadingHandler, html, options)
    }

    #[test]
    fn test_h1_atx() {
        let result = convert_test("<h1>Hello</h1>", &ConversionOptions::default());
        assert!(result.contains("# Hello"));
    }

    #[test]
    fn test_h2_atx() {
        let result = convert_test("<h2>World</h2>", &ConversionOptions::default());
        assert!(result.contains("## World"));
    }

    #[test]
    fn test_h1_setext() {
        let options = ConversionOptions::new().heading_style(HeadingStyle::Setext);
        let result = convert_test("<h1>Title</h1>", &options);
        assert!(result.contains("Title\n====="));
    }

    #[test]
    fn test_h2_setext() {
        let options = ConversionOptions::new().heading_style(HeadingStyle::Setext);
        let result = convert_test("<h2>Subtitle</h2>", &options);
        assert!(result.contains("Subtitle\n--------"));
    }

    #[test]
    fn test_empty_heading() {
        let result = convert_test("<h1></h1>", &ConversionOptions::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_whitespace_normalization() {
        let result = convert_test("<h1>Hello   World</h1>", &ConversionOptions::default());
        assert!(result.contains("# Hello World"));
    }
}
