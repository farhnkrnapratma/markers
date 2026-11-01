use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};

pub struct AbbreviationHandler;

impl TagHandler for AbbreviationHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["abbr"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let content = cx.children(element);
        let content = content.trim();
        if content.is_empty() {
            return String::new();
        }

        match element.value().attr("title") {
            Some(title) => format!(
                "<abbr title=\"{}\">{}</abbr>",
                escape_html_attribute(title),
                content
            ),
            None => format!("<abbr>{}</abbr>", content),
        }
    }
}

fn escape_html_attribute(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render;

    #[test]
    fn test_abbr() {
        assert_eq!(
            render(&AbbreviationHandler, "<abbr>HTML</abbr>"),
            "<abbr>HTML</abbr>"
        );
    }

    #[test]
    fn test_abbr_with_title() {
        assert_eq!(
            render(
                &AbbreviationHandler,
                r#"<abbr title="HyperText Markup Language">HTML</abbr>"#
            ),
            "<abbr title=\"HyperText Markup Language\">HTML</abbr>"
        );
    }

    #[test]
    fn test_abbr_title_is_attribute_escaped() {
        assert_eq!(
            render(
                &AbbreviationHandler,
                r#"<abbr title="a &quot;b&quot; &lt;c&gt; &amp; d">X</abbr>"#
            ),
            "<abbr title=\"a &quot;b&quot; &lt;c&gt; &amp; d\">X</abbr>"
        );
    }

    #[test]
    fn test_empty_abbr_renders_nothing() {
        assert!(render(&AbbreviationHandler, "<abbr></abbr>").is_empty());
    }
}
