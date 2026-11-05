use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};

#[derive(Debug, Clone, Copy)]
pub struct WrapHandler {
    tags: &'static [&'static str],
    open: &'static str,
    close: &'static str,
}

macro_rules! wrap {
    ($tags:expr, $open:expr, $close:expr) => {
        WrapHandler {
            tags: $tags,
            open: $open,
            close: $close,
        }
    };
}

impl WrapHandler {
    pub const PARAGRAPH: Self = wrap!(&["p"], "\n\n", "\n\n");
    pub const STRONG: Self = wrap!(&["strong", "b"], "**", "**");
    pub const EMPHASIS: Self = wrap!(&["em", "i"], "*", "*");
    pub const STRIKETHROUGH: Self = wrap!(&["del", "s", "strike"], "~~", "~~");
    pub const SUBSCRIPT: Self = wrap!(&["sub"], "<sub>", "</sub>");
    pub const SUPERSCRIPT: Self = wrap!(&["sup"], "<sup>", "</sup>");
    pub const KEYBOARD: Self = wrap!(&["kbd"], "<kbd>", "</kbd>");
    pub const MARK: Self = wrap!(&["mark"], "<mark>", "</mark>");
    pub const SAMPLE_OUTPUT: Self = wrap!(&["samp"], "<samp>", "</samp>");
    pub const VARIABLE: Self = wrap!(&["var"], "<var>", "</var>");
}

impl TagHandler for WrapHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        self.tags
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let content = cx.children(element);
        let content = content.trim();
        if content.is_empty() {
            return String::new();
        }
        format!("{}{}{}", self.open, content, self.close)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render;

    #[test]
    fn test_paragraph() {
        let result = render(&WrapHandler::PARAGRAPH, "<p>Hello World</p>");
        assert!(result.contains("Hello World"));
        assert!(result.starts_with("\n\n"));
        assert!(result.ends_with("\n\n"));
    }

    #[test]
    fn test_empty_paragraph() {
        assert!(render(&WrapHandler::PARAGRAPH, "<p></p>").is_empty());
    }

    #[test]
    fn test_whitespace_paragraph() {
        assert!(render(&WrapHandler::PARAGRAPH, "<p>   </p>").is_empty());
    }

    #[test]
    fn test_strong() {
        assert_eq!(
            render(&WrapHandler::STRONG, "<strong>bold</strong>"),
            "**bold**"
        );
        assert_eq!(render(&WrapHandler::STRONG, "<b>bold</b>"), "**bold**");
    }

    #[test]
    fn test_emphasis() {
        assert_eq!(
            render(&WrapHandler::EMPHASIS, "<em>italic</em>"),
            "*italic*"
        );
        assert_eq!(render(&WrapHandler::EMPHASIS, "<i>italic</i>"), "*italic*");
    }

    #[test]
    fn test_empty_strong_and_emphasis() {
        assert_eq!(render(&WrapHandler::STRONG, "<strong></strong>"), "");
        assert_eq!(render(&WrapHandler::EMPHASIS, "<em></em>"), "");
    }

    #[test]
    fn test_strikethrough_tags() {
        for html in ["<del>x</del>", "<s>x</s>", "<strike>x</strike>"] {
            assert_eq!(render(&WrapHandler::STRIKETHROUGH, html), "~~x~~", "{html}");
        }
        assert_eq!(render(&WrapHandler::STRIKETHROUGH, "<del></del>"), "");
    }

    #[test]
    fn test_subscript_and_superscript() {
        assert_eq!(
            render(&WrapHandler::SUBSCRIPT, "<sub>2</sub>"),
            "<sub>2</sub>"
        );
        assert_eq!(
            render(&WrapHandler::SUPERSCRIPT, "<sup>2</sup>"),
            "<sup>2</sup>"
        );
        assert_eq!(render(&WrapHandler::SUBSCRIPT, "<sub></sub>"), "");
        assert_eq!(render(&WrapHandler::SUPERSCRIPT, "<sup></sup>"), "");
    }

    #[test]
    fn test_html_passthrough_tags_keep_their_markup() {
        assert_eq!(
            render(&WrapHandler::KEYBOARD, "<kbd>Ctrl+C</kbd>"),
            "<kbd>Ctrl+C</kbd>"
        );
        assert_eq!(
            render(&WrapHandler::MARK, "<mark>highlighted</mark>"),
            "<mark>highlighted</mark>"
        );
        assert_eq!(
            render(&WrapHandler::SAMPLE_OUTPUT, "<samp>output</samp>"),
            "<samp>output</samp>"
        );
        assert_eq!(
            render(&WrapHandler::VARIABLE, "<var>x</var>"),
            "<var>x</var>"
        );
    }

    #[test]
    fn test_empty_passthrough_tags_render_nothing() {
        assert!(render(&WrapHandler::KEYBOARD, "<kbd></kbd>").is_empty());
        assert!(render(&WrapHandler::MARK, "<mark></mark>").is_empty());
        assert!(render(&WrapHandler::SAMPLE_OUTPUT, "<samp></samp>").is_empty());
        assert!(render(&WrapHandler::VARIABLE, "<var></var>").is_empty());
    }
}
