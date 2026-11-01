mod annotate;
mod cleanup;
mod input_guard;
mod render;

use scraper::Html;

use crate::handlers::HandlerRegistry;
use crate::options::ConversionOptions;

use cleanup::finalize_markdown;
use input_guard::{exceeds_nesting_limit, strip_markup, MAX_INPUT_NESTING};
use render::RenderPass;

pub use annotate::Annotations;
pub(crate) use annotate::{annotate_tree, ElementFilter};

pub struct Converter {
    handlers: HandlerRegistry,
}

impl Converter {
    pub fn new() -> Self {
        Self {
            handlers: HandlerRegistry::standard(),
        }
    }

    pub fn convert(&self, html: &str, options: &ConversionOptions) -> String {
        if html.is_empty() {
            return String::new();
        }

        if exceeds_nesting_limit(html, MAX_INPUT_NESTING) {
            return self.convert(&strip_markup(html), options);
        }

        let dom = Html::parse_document(html);
        let filter = ElementFilter::new(options);
        let annotations = annotate_tree(&dom, &filter, options);

        let markdown =
            RenderPass::new(&self.handlers, &annotations, options).element(dom.root_element());

        finalize_markdown(markdown, options)
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{HeadingStyle, LinkStyle};
    use crate::pipeline::annotate::MAX_RENDER_DEPTH;

    fn convert(html: &str) -> String {
        Converter::new().convert(html, &ConversionOptions::default())
    }

    fn convert_with(html: &str, options: &ConversionOptions) -> String {
        Converter::new().convert(html, options)
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(convert(""), "");
    }

    #[test]
    fn test_heading() {
        let result = convert("<h1>Title</h1>");
        assert!(result.contains("# Title"));
    }

    #[test]
    fn test_paragraph() {
        let result = convert("<p>Hello World</p>");
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_link() {
        let result = convert(r#"<a href="https://example.com">Link</a>"#);
        assert!(result.contains("[Link](https://example.com)"));
    }

    #[test]
    fn test_image() {
        let result = convert(r#"<img src="image.png" alt="Alt text">"#);
        assert!(result.contains("![Alt text](image.png)"));
    }

    #[test]
    fn test_emphasis() {
        let result = convert("<em>italic</em>");
        assert!(result.contains("*italic*"));
    }

    #[test]
    fn test_strong() {
        let result = convert("<strong>bold</strong>");
        assert!(result.contains("**bold**"));
    }

    #[test]
    fn test_code() {
        let result = convert("<code>inline</code>");
        assert!(result.contains("`inline`"));
    }

    #[test]
    fn test_pre() {
        let result = convert("<pre>code block</pre>");
        assert!(result.contains("```"));
        assert!(result.contains("code block"));
    }

    #[test]
    fn test_list() {
        let result = convert("<ul><li>One</li><li>Two</li></ul>");
        assert!(result.contains("- One"));
        assert!(result.contains("- Two"));
    }

    #[test]
    fn test_ordered_list() {
        let result = convert("<ol><li>First</li><li>Second</li></ol>");
        assert!(result.contains("1. First"));
        assert!(result.contains("2. Second"));
    }

    #[test]
    fn test_blockquote() {
        let result = convert("<blockquote>Quote</blockquote>");
        assert!(result.contains("> Quote"));
    }

    #[test]
    fn test_nested_blockquotes() {
        let result = convert("<blockquote>Outer<blockquote>Inner</blockquote></blockquote>");
        assert!(result.contains("> Outer"));
        assert!(result.contains("> > Inner"));
    }

    #[test]
    fn test_deeply_nested_blockquotes() {
        let result = convert(
        "<blockquote>Level 1<blockquote>Level 2<blockquote>Level 3</blockquote></blockquote></blockquote>",
    );
        assert!(result.contains("> Level 1"));
        assert!(result.contains("> > Level 2"));
        assert!(result.contains("> > > Level 3"));
    }

    #[test]
    fn test_hr() {
        let result = convert("<hr>");
        assert!(result.contains("---"));
    }

    #[test]
    fn test_entity_decoding() {
        let result = convert("<p>&lt;html&gt; &amp; more</p>");
        assert!(result.contains("<html> & more"));
    }

    #[test]
    fn test_nested_elements() {
        let result = convert("<p>This is <strong>bold and <em>italic</em></strong> text.</p>");
        assert!(result.contains("**bold and *italic***"));
    }

    #[test]
    fn test_setext_headings() {
        let options = ConversionOptions::new().heading_style(HeadingStyle::Setext);
        let result = convert_with("<h1>Title</h1>", &options);
        assert!(result.contains("====="));
    }

    #[test]
    fn test_referenced_links() {
        let options = ConversionOptions::new().link_style(LinkStyle::Referenced);
        let result = convert_with(
            r#"<p><a href="https://a.com">A</a> and <a href="https://b.com">B</a></p>"#,
            &options,
        );
        assert!(result.contains("[A][1]"));
        assert!(result.contains("[B][2]"));
        assert!(result.contains("[1]: https://a.com"));
    }

    #[test]
    fn test_exclude_selector() {
        let options = ConversionOptions::new().exclude_selectors(vec!["nav".to_string()]);
        let result = convert_with("<div><nav>Skip this</nav><p>Keep this</p></div>", &options);
        assert!(!result.contains("Skip this"));
        assert!(result.contains("Keep this"));
    }

    #[test]
    fn test_whitespace_normalization() {
        let result = convert("<p>Hello    world</p>");
        assert!(result.contains("Hello world"));

        let result = convert("<p>Hello\n\n\nworld</p>");
        assert!(result.contains("Hello world"));

        let result = convert("<p>Hello\t\tworld</p>");
        assert!(result.contains("Hello world"));

        let result = convert("<p>Hello  \n\t  world</p>");
        assert!(result.contains("Hello world"));
    }

    #[test]
    fn test_complex_document() {
        let html = r#"
            <article>
                <h1>Article Title</h1>
                <p>First paragraph with <strong>bold</strong> and <em>italic</em>.</p>
                <h2>Section</h2>
                <p>A <a href="https://example.com">link</a> here.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
                <pre><code class="language-rust">fn main() {}</code></pre>
            </article>
        "#;
        let result = convert(html);

        assert!(result.contains("# Article Title"));
        assert!(result.contains("## Section"));
        assert!(result.contains("**bold**"));
        assert!(result.contains("*italic*"));
        assert!(result.contains("[link](https://example.com)"));
        assert!(result.contains("- Item 1"));
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_nesting_beyond_depth_cap_keeps_text() {
        let n = MAX_RENDER_DEPTH * 4;
        let html = "<blockquote>".repeat(n) + "deep text" + &"</blockquote>".repeat(n);
        let result = Converter::new().convert(&html, &ConversionOptions::default());
        assert!(result.contains("deep text"));
    }

    #[test]
    fn test_pathological_nesting_is_flattened_to_text() {
        let n = MAX_INPUT_NESTING * 20;
        let html = "<div>".repeat(n) + "deep text" + &"</div>".repeat(n);
        let result = Converter::new().convert(&html, &ConversionOptions::default());
        assert_eq!(result.trim(), "deep text");
    }

    #[test]
    fn test_depth_cap_fits_in_small_stack() {
        let n = MAX_RENDER_DEPTH * 2;
        let cases = [
            ("<blockquote>".repeat(n), "</blockquote>".repeat(n)),
            ("<ul><li>".repeat(n), "</li></ul>".repeat(n)),
            (
                "<details><summary>s</summary>".repeat(n),
                "</details>".repeat(n),
            ),
            ("<dl><dd>".repeat(n), "</dd></dl>".repeat(n)),
            ("<table><tr><td>".repeat(n), "</td></tr></table>".repeat(n)),
        ];
        for (open, close) in cases {
            let html = format!("{open}deep text{close}");
            let result = std::thread::Builder::new()
                .stack_size(1024 * 1024)
                .spawn(move || Converter::new().convert(&html, &ConversionOptions::default()))
                .unwrap()
                .join()
                .expect("conversion overflowed a 1 MiB stack");
            assert!(result.contains("deep text"));
        }
    }

    #[test]
    fn test_nested_table_in_cell_is_flattened() {
        let html = "<table><tr><td>outer <table><tr><td>inner</td></tr></table></td></tr></table>";
        let result = Converter::new().convert(html, &ConversionOptions::default());
        assert!(result.contains("outer"));
        assert!(result.contains("inner"));
        assert_eq!(result.matches('|').count(), 4, "{result}");
    }

    #[test]
    fn test_nested_tables_output_stays_linear() {
        let n = 12;
        let html = format!(
            "{}deep text{}",
            "<table><tr><td>".repeat(n),
            "</td></tr></table>".repeat(n)
        );
        let result = Converter::new().convert(&html, &ConversionOptions::default());
        assert!(result.contains("deep text"));
        assert!(
            result.len() < 1_000,
            "output grew to {} bytes",
            result.len()
        );
    }

    fn convert_default(html: &str) -> String {
        Converter::new().convert(html, &ConversionOptions::default())
    }

    #[test]
    fn test_text_cannot_form_a_link_or_image() {
        assert_eq!(
            convert_default("<p>[click](javascript:alert(1))</p>"),
            r"\[click\](javascript:alert(1))"
        );
        assert_eq!(
            convert_default("<p>![x](javascript:alert(1))</p>"),
            r"!\[x\](javascript:alert(1))"
        );
        assert_eq!(
            convert_default("<p>[x]: javascript:alert(1)</p>"),
            r"\[x\]: javascript:alert(1)"
        );
    }

    #[test]
    fn test_text_cannot_form_raw_html_or_autolink() {
        assert_eq!(
            convert_default("<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>"),
            r"\<script>alert(1)\</script>"
        );
        assert_eq!(
            convert_default("<p>&lt;javascript:alert(1)&gt;</p>"),
            r"\<javascript:alert(1)>"
        );
        assert_eq!(
            convert_default("<p>1 &lt; 2 and 3 &gt; 2</p>"),
            "1 < 2 and 3 > 2"
        );
    }

    #[test]
    fn test_backslash_cannot_cancel_an_escape() {
        assert_eq!(convert_default(r"<p>a\[b</p>"), r"a\\\[b");
        assert_eq!(
            convert_default(
                r"<span>\</span><span>[x</span><span>\</span><span>](javascript:alert(1))</span>"
            ),
            r"\\\[x\\\](javascript:alert(1))"
        );
    }

    #[test]
    fn test_code_and_link_text_keep_expected_form() {
        assert_eq!(convert_default("<p><code>a[0]</code></p>"), "`a[0]`");
        assert_eq!(
            convert_default("<pre><code>x[1] = \\n;</code></pre>").trim(),
            "```\nx[1] = \\n;\n```"
        );
        assert_eq!(
            convert_default(r#"<a href="https://e.com">a[1]</a>"#),
            r"[a\[1\]](https://e.com)"
        );
    }

    #[test]
    fn test_flattened_nested_table_text_is_escaped() {
        let html = "<table><tr><td><table><tr><td>[x](javascript:alert(1))</td></tr></table></td></tr></table>";
        let result = convert_default(html);
        assert!(result.contains(r"\[x\](javascript:alert(1))"), "{result}");
    }

    #[test]
    fn test_entities_are_decoded_exactly_once() {
        assert_eq!(
            convert_default("<p>&amp;lt;b&amp;gt; &amp;amp; &amp;nbsp; &amp;#169;</p>"),
            "&lt;b&gt; &amp; &nbsp; &#169;"
        );
        assert_eq!(
            convert_default("<p>AT&amp;T &lt;3 &copy; &#169; &#x41; &hellip; &rarr;</p>"),
            "AT&T <3 © © A … →"
        );
    }
}
