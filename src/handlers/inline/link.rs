use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};
use crate::text::escape::{escape_inline_text, escape_link_destination, escape_link_title};
use crate::text::url::{has_unsafe_scheme, resolve_against_base};
use crate::text::whitespace::collapse_whitespace;

pub struct LinkHandler;

fn can_be_autolink(destination: &str) -> bool {
    !destination
        .chars()
        .any(|c| matches!(c, '<' | '>') || c.is_whitespace() || c.is_control())
}

impl TagHandler for LinkHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["a"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let href = element.value().attr("href").unwrap_or("");
        let title = element.value().attr("title");

        let raw_content = cx.children(element);
        let raw_content = collapse_whitespace(raw_content.trim());

        let content = if raw_content.is_empty() {
            if let Some(label) = element
                .value()
                .attr("title")
                .or_else(|| element.value().attr("aria-label"))
            {
                escape_inline_text(&collapse_whitespace(label.trim()))
            } else {
                raw_content.to_string()
            }
        } else {
            raw_content.to_string()
        };

        if href.is_empty() || href == "#" || has_unsafe_scheme(href) {
            return content.to_string();
        }

        let href = if let Some(base) = &cx.options.base_url {
            resolve_against_base(base, href)
        } else {
            href.to_string()
        };

        if title.is_none() {
            if let Some(email) = href.strip_prefix("mailto:") {
                if content == email && can_be_autolink(email) {
                    return format!("<{}>", email);
                }
            }
            if content == href && can_be_autolink(&href) {
                return format!("<{}>", href);
            }
        }

        let href = escape_link_destination(&href);

        match title {
            Some(t) => {
                format!("[{}]({} \"{}\")", content, href, escape_link_title(t))
            }
            None => {
                format!("[{}]({})", content, href)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render_with;
    use crate::options::ConversionOptions;

    fn convert_test(html: &str, options: &ConversionOptions) -> String {
        render_with(&LinkHandler, html, options)
    }

    #[test]
    fn test_simple_link() {
        let result = convert_test(
            r#"<a href="https://example.com">Link</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "[Link](https://example.com)");
    }

    #[test]
    fn test_link_with_title() {
        let result = convert_test(
            r#"<a href="https://example.com" title="Example">Link</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, r#"[Link](https://example.com "Example")"#);
    }

    #[test]
    fn test_empty_href() {
        let result = convert_test(r#"<a href="">Text</a>"#, &ConversionOptions::default());
        assert_eq!(result, "Text");
    }

    #[test]
    fn test_fragment_only() {
        let result = convert_test(r##"<a href="#">Text</a>"##, &ConversionOptions::default());
        assert_eq!(result, "Text");
    }

    #[test]
    fn test_relative_url_with_base() {
        let options =
            ConversionOptions::new().base_url(Some("https://example.com/dir/".to_string()));
        let result = convert_test(r#"<a href="page.html">Link</a>"#, &options);
        assert!(result.contains("https://example.com/dir/page.html"));
    }

    #[test]
    fn test_url_with_spaces() {
        let result = convert_test(
            r#"<a href="https://example.com/path with spaces">Link</a>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("%20"));
    }

    #[test]
    fn test_autolink_url() {
        let result = convert_test(
            r#"<a href="https://example.com">https://example.com</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "<https://example.com>");
    }

    #[test]
    fn test_autolink_email() {
        let result = convert_test(
            r#"<a href="mailto:test@example.com">test@example.com</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "<test@example.com>");
    }

    #[test]
    fn test_no_autolink_when_text_differs() {
        let result = convert_test(
            r#"<a href="https://example.com">Example Site</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "[Example Site](https://example.com)");
    }

    #[test]
    fn test_no_autolink_with_title() {
        let result = convert_test(
            r#"<a href="https://example.com" title="tip">https://example.com</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(
            result,
            r#"[https://example.com](https://example.com "tip")"#
        );
    }

    #[test]
    fn test_fragment_link_with_section() {
        let result = convert_test(
            r##"<a href="#section">Section</a>"##,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "[Section](#section)");
    }

    #[test]
    fn test_fragment_link_with_complex_id() {
        let result = convert_test(
            r##"<a href="#user-guide-installation">Installation</a>"##,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "[Installation](#user-guide-installation)");
    }

    #[test]
    fn test_empty_link_text_uses_title() {
        let result = convert_test(
            r#"<a href="https://example.com" title="Home"></a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, r#"[Home](https://example.com "Home")"#);
    }

    #[test]
    fn test_empty_link_text_uses_aria_label() {
        let result = convert_test(
            r#"<a href="https://example.com" aria-label="Settings"></a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "[Settings](https://example.com)");
    }

    #[test]
    fn test_empty_link_text_title_takes_precedence_over_aria_label() {
        let result = convert_test(
            r#"<a href="https://example.com" title="Title" aria-label="Aria"></a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, r#"[Title](https://example.com "Title")"#);
    }

    #[test]
    fn test_non_empty_link_text_ignores_fallbacks() {
        let result = convert_test(
            r#"<a href="https://example.com" title="Title" aria-label="Aria">Real Text</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, r#"[Real Text](https://example.com "Title")"#);
    }

    #[test]
    fn test_javascript_link_rendered_as_text() {
        let result = convert_test(
            r#"<a href="javascript:alert(1)">click</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "click");
    }

    #[test]
    fn test_obfuscated_dangerous_link_rendered_as_text() {
        for href in [
            "JAVASCRIPT:alert(1)",
            " javascript:alert(1)",
            "java&#9;script:alert(1)",
            "vbscript:x",
            "data:text/html,<script>x</script>",
        ] {
            let html = format!(r#"<a href="{href}">click</a>"#);
            assert_eq!(
                convert_test(&html, &ConversionOptions::default()),
                "click",
                "{href}"
            );
        }
    }

    #[test]
    fn test_dangerous_link_with_base_url_still_rendered_as_text() {
        let options = ConversionOptions::new().base_url(Some("https://example.com/".to_string()));
        let result = convert_test(r#"<a href="javascript:alert(1)">click</a>"#, &options);
        assert_eq!(result, "click");
    }

    #[test]
    fn test_link_url_with_angle_brackets_and_newline_is_escaped() {
        let result = convert_test(
            "<a href=\"https://example.com/a<b>\n\">x</a>",
            &ConversionOptions::default(),
        );
        assert_eq!(result, "[x](https://example.com/a%3Cb%3E%0A)");
    }

    #[test]
    fn test_fallback_text_from_title_is_escaped() {
        let result = convert_test(
            r#"<a href="https://e.com" title="](javascript:alert(1))[x"></a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(
            result,
            r#"[\](javascript:alert(1))\[x](https://e.com "](javascript:alert(1))[x")"#
        );
    }

    #[test]
    fn test_fallback_text_from_aria_label_is_escaped() {
        let result = convert_test(
            r#"<a href="https://e.com" aria-label="a&lt;script&gt;b"></a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, r"[a\<script>b](https://e.com)");
    }

    #[test]
    fn test_autolink_is_not_used_for_destinations_it_cannot_hold() {
        let result = convert_test(
            r#"<a href="https://e.com/a b">https://e.com/a b</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "[https://e.com/a b](https://e.com/a%20b)");

        let result = convert_test(
            r#"<a href="mailto:a b@c.com">a b@c.com</a>"#,
            &ConversionOptions::default(),
        );
        assert!(!result.starts_with('<'), "{result}");
    }

    #[test]
    fn test_plain_url_still_becomes_an_autolink() {
        let result = convert_test(
            r#"<a href="https://e.com/x">https://e.com/x</a>"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "<https://e.com/x>");
    }
}
