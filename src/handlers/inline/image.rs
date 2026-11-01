use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};
use crate::text::escape::{escape_image_alt, escape_link_destination, escape_link_title};
use crate::text::url::{has_unsafe_scheme, resolve_against_base};

pub struct ImageHandler;

impl TagHandler for ImageHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["img"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let src = element.value().attr("src").unwrap_or("");
        let alt = element.value().attr("alt").unwrap_or("");
        let title = element.value().attr("title");

        if src.is_empty() || has_unsafe_scheme(src) {
            return String::new();
        }

        let src = if let Some(base) = &cx.options.base_url {
            resolve_against_base(base, src)
        } else {
            src.to_string()
        };

        let src = escape_link_destination(&src);
        let alt = escape_image_alt(alt);

        match title {
            Some(t) => format!("![{}]({} \"{}\")", alt, src, escape_link_title(t)),
            None => format!("![{}]({})", alt, src),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render_with;
    use crate::options::ConversionOptions;

    fn convert_test(html: &str, options: &ConversionOptions) -> String {
        render_with(&ImageHandler, html, options)
    }

    #[test]
    fn test_simple_image() {
        let result = convert_test(
            r#"<img src="image.png" alt="An image">"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "![An image](image.png)");
    }

    #[test]
    fn test_image_with_title() {
        let result = convert_test(
            r#"<img src="image.png" alt="Alt" title="Title">"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, r#"![Alt](image.png "Title")"#);
    }

    #[test]
    fn test_empty_src() {
        let result = convert_test(
            r#"<img src="" alt="No source">"#,
            &ConversionOptions::default(),
        );
        assert!(result.is_empty());
    }

    #[test]
    fn test_no_alt() {
        let result = convert_test(r#"<img src="image.png">"#, &ConversionOptions::default());
        assert_eq!(result, "![](image.png)");
    }

    #[test]
    fn test_relative_url_with_base() {
        let options = ConversionOptions::new().base_url(Some("https://example.com/".to_string()));
        let result = convert_test(r#"<img src="images/photo.jpg" alt="Photo">"#, &options);
        assert!(result.contains("https://example.com/images/photo.jpg"));
    }

    #[test]
    fn test_base64_image_filtered() {
        let result = convert_test(
            r#"<img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUg..." alt="Icon">"#,
            &ConversionOptions::default(),
        );
        assert!(
            result.is_empty(),
            "base64 image should be filtered, got: {}",
            result
        );
    }

    #[test]
    fn test_data_svg_filtered() {
        let result = convert_test(
            r#"<img src="data:image/svg+xml;base64,PHN2Zy..." alt="SVG">"#,
            &ConversionOptions::default(),
        );
        assert!(result.is_empty());
    }

    #[test]
    fn test_normal_image_not_filtered() {
        let result = convert_test(
            r#"<img src="https://example.com/photo.jpg" alt="Photo">"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "![Photo](https://example.com/photo.jpg)");
    }

    #[test]
    fn test_javascript_image_src_filtered() {
        let result = convert_test(
            r#"<img src="javascript:alert(1)" alt="x">"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "");
    }

    #[test]
    fn test_uppercase_data_scheme_filtered() {
        let result = convert_test(
            r#"<img src="DATA:image/png;base64,AAAA" alt="x">"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, "");
    }

    #[test]
    fn test_image_alt_cannot_close_the_link() {
        let result = convert_test(
            r#"<img src="/ok.png" alt="a](javascript:alert(1))[b">"#,
            &ConversionOptions::default(),
        );
        assert_eq!(result, r"![a\](javascript:alert(1))\[b](/ok.png)");
    }
}
