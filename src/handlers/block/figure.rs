use scraper::ElementRef;

use crate::handlers::inline::ImageHandler;
use crate::handlers::{RenderContext, TagHandler};
use crate::text::whitespace::collapse_whitespace;

pub struct FigureHandler;

impl TagHandler for FigureHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["figure"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let mut image_md = String::new();
        let mut caption = String::new();

        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                let tag = el.value().name();
                match tag {
                    "img" => {
                        let md = ImageHandler.render(el, cx);
                        if !md.is_empty() {
                            image_md = md;
                        }
                    }
                    "figcaption" => {
                        let c = cx.children(el);
                        caption = collapse_whitespace(c.trim());
                    }
                    "picture" => {
                        for pic_child in el.children() {
                            if let Some(pic_el) = ElementRef::wrap(pic_child) {
                                if pic_el.value().name() == "img" {
                                    let md = ImageHandler.render(pic_el, cx);
                                    if !md.is_empty() {
                                        image_md = md;
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        let nested = cx.children(el);
                        if image_md.is_empty() && nested.contains("![") {
                            image_md = nested;
                        }
                    }
                }
            }
        }

        if image_md.is_empty() {
            return String::new();
        }

        let mut result = format!("\n\n{}", image_md.trim());
        if !caption.is_empty() {
            result.push_str(&format!("\n*{}*", caption));
        }
        result.push_str("\n\n");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::{render, render_with};
    use crate::options::ConversionOptions;

    fn convert_test(html: &str) -> String {
        render(&FigureHandler, html)
    }

    #[test]
    fn test_figure_with_caption() {
        let result = convert_test(
            r#"<figure>
                <img src="photo.jpg" alt="A photo">
                <figcaption>This is the caption</figcaption>
            </figure>"#,
        );
        assert!(result.contains("![A photo](photo.jpg)"));
        assert!(result.contains("*This is the caption*"));
    }

    #[test]
    fn test_figure_without_caption() {
        let result = convert_test(
            r#"<figure>
                <img src="photo.jpg" alt="A photo">
            </figure>"#,
        );
        assert!(result.contains("![A photo](photo.jpg)"));
        assert!(!result.contains("*"));
    }

    #[test]
    fn test_figure_with_picture() {
        let result = convert_test(
            r#"<figure>
                <picture>
                    <source srcset="photo.webp" type="image/webp">
                    <img src="photo.jpg" alt="A photo">
                </picture>
            </figure>"#,
        );
        assert!(result.contains("![A photo](photo.jpg)"));
    }

    #[test]
    fn test_empty_figure() {
        let result = convert_test("<figure></figure>");
        assert!(result.is_empty());
    }

    #[test]
    fn test_figure_drops_unsafe_image_url() {
        for html in [
            r#"<figure><img src="javascript:alert(1)" alt="x"></figure>"#,
            r#"<figure><picture><img src="data:text/html,hi" alt="x"></picture></figure>"#,
        ] {
            assert_eq!(convert_test(html), "", "{html}");
        }
    }

    #[test]
    fn test_figure_escapes_image_url() {
        let result = convert_test(r#"<figure><img src="a b).png" alt="x"></figure>"#);
        assert!(result.contains("![x](a%20b%29.png)"), "{result}");
    }

    #[test]
    fn test_figure_resolves_base_url() {
        let options = ConversionOptions::new().base_url(Some("https://example.com".to_string()));
        let result = render_with(
            &FigureHandler,
            r#"<figure><img src="/rel.png" alt="x"></figure>"#,
            &options,
        );
        assert!(
            result.contains("![x](https://example.com/rel.png)"),
            "{result}"
        );
    }
}
