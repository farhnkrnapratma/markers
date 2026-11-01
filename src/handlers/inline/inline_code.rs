use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};
use crate::text::escape::backtick_delimiter_len;

pub struct InlineCodeHandler;

impl TagHandler for InlineCodeHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["code"]
    }

    fn render(&self, element: ElementRef, _cx: &RenderContext) -> String {
        if let Some(parent) = element.parent() {
            if let Some(parent_el) = ElementRef::wrap(parent) {
                if parent_el.value().name() == "pre" {
                    return element.text().collect::<Vec<_>>().join("");
                }
            }
        }

        let code = element.text().collect::<Vec<_>>().join("");

        if code.is_empty() {
            return String::new();
        }

        let backticks = backtick_delimiter_len(&code);
        let delim = "`".repeat(backticks);

        let (prefix, suffix) = if code.starts_with('`') || code.ends_with('`') {
            (" ", " ")
        } else {
            ("", "")
        };

        format!("{}{}{}{}{}", delim, prefix, code, suffix, delim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render;

    fn convert_test(html: &str) -> String {
        render(&InlineCodeHandler, html)
    }

    #[test]
    fn test_simple_code() {
        assert_eq!(convert_test("<code>hello</code>"), "`hello`");
    }

    #[test]
    fn test_code_with_backticks() {
        assert_eq!(convert_test("<code>use `this`</code>"), "`` use `this` ``");
    }

    #[test]
    fn test_code_starting_with_backtick() {
        assert_eq!(convert_test("<code>`start</code>"), "`` `start ``");
    }

    #[test]
    fn test_empty_code() {
        assert_eq!(convert_test("<code></code>"), "");
    }
}
