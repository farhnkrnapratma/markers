mod options;
mod pipeline;
mod text;

mod handlers;

#[cfg(any(feature = "napi", feature = "wasm"))]
mod bindings;

pub use options::{ConversionOptions, HeadingStyle, LinkStyle};
use pipeline::Converter;

pub fn convert(html: &str) -> String {
    convert_with_options(html, &ConversionOptions::default())
}

pub fn convert_with_options(html: &str, options: &ConversionOptions) -> String {
    let converter = Converter::new();
    converter.convert(html, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() {
        let html = "<h1>Hello World</h1>";
        let markdown = convert(html);
        assert!(markdown.contains("# Hello World"));
    }

    #[test]
    fn test_empty_input() {
        let markdown = convert("");
        assert!(markdown.is_empty());
    }

    #[test]
    fn test_paragraph() {
        let html = "<p>This is a paragraph.</p>";
        let markdown = convert(html);
        assert!(markdown.contains("This is a paragraph."));
    }

    #[test]
    fn test_multiple_elements() {
        let html = "<h1>Title</h1><p>First paragraph.</p><p>Second paragraph.</p>";
        let markdown = convert(html);
        assert!(markdown.contains("# Title"));
        assert!(markdown.contains("First paragraph."));
        assert!(markdown.contains("Second paragraph."));
    }
}
