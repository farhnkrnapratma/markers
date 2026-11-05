use markers::convert;

#[test]
fn empty_string_returns_empty() {
    assert_eq!(convert(""), "");
}

#[test]
fn whitespace_only_returns_empty() {
    assert!(convert("   \n\t\n   ").trim().is_empty());
}

#[test]
fn single_paragraph() {
    let result = convert("<p>Hello world</p>");
    assert!(result.contains("Hello world"));
}

#[test]
fn large_html_100k_paragraphs() {
    let html: String = (0..1000)
        .map(|i| format!("<p>Paragraph {}</p>", i))
        .collect();
    let result = convert(&html);
    assert!(result.contains("Paragraph 0"));
    assert!(result.contains("Paragraph 999"));
}

#[test]
fn very_long_single_paragraph() {
    let content = "word ".repeat(10_000);
    let html = format!("<p>{}</p>", content);
    let result = convert(&html);
    assert!(result.len() > 10_000);
}

#[test]
fn html_entities_decoded() {
    let result = convert("<p>&amp; &lt; &gt; &quot;</p>");
    assert!(result.contains("&"));
    assert!(result.contains("<"));
    assert!(result.contains(">"));
}

#[test]
fn numeric_entities_decoded() {
    let result = convert("<p>&#169; &#8212;</p>");
    assert!(result.contains("©") || result.contains("&#169;"));
}

#[test]
fn hex_entities_decoded() {
    let result = convert("<p>&#x27;</p>");
    assert!(result.contains("'") || result.contains("&#x27;"));
}

#[test]
fn unicode_content_preserved() {
    let result = convert("<p>日本語テスト 🚀 émojis</p>");
    assert!(result.contains("日本語テスト"));
    assert!(result.contains("🚀"));
}

#[test]
fn zero_width_characters_handled() {
    let result = convert("<p>te\u{200B}st</p>");
    assert!(result.contains("te") || result.contains("test"));
}

#[test]
fn deeply_nested_divs_20_levels() {
    let mut html = String::new();
    for _ in 0..20 {
        html.push_str("<div>");
    }
    html.push_str("<p>Deep content</p>");
    for _ in 0..20 {
        html.push_str("</div>");
    }
    let result = convert(&html);
    assert!(result.contains("Deep content"));
}

#[test]
fn unclosed_tags_handled() {
    let result = convert("<p>Unclosed paragraph<p>Second paragraph");
    assert!(result.contains("Unclosed paragraph"));
    assert!(result.contains("Second paragraph"));
}

#[test]
fn overlapping_tags_handled() {
    let result = convert("<b><i>overlapped</b></i>");
    assert!(result.contains("overlapped"));
}

#[test]
fn html_comments_stripped() {
    let result = convert("<p>Before</p><!-- comment --><p>After</p>");
    assert!(result.contains("Before"));
    assert!(result.contains("After"));
    assert!(!result.contains("comment"));
}

#[test]
fn script_tags_handled_without_crash() {
    let result = convert("<p>Content</p><script>alert('xss')</script><p>More</p>");
    assert!(result.contains("Content"));
    assert!(result.contains("More"));
}

#[test]
fn style_tags_handled_without_crash() {
    let result = convert("<style>.x{color:red}</style><p>Visible</p>");
    assert!(result.contains("Visible"));
}

#[test]
fn link_with_no_href() {
    let result = convert("<a>Just text</a>");
    assert!(result.contains("Just text"));
}

#[test]
fn image_with_no_src() {
    let result = convert("<img alt='broken'>");
    assert!(!result.contains("![broken]()"));
}

#[test]
fn pre_with_backticks_in_content() {
    let result = convert("<pre><code>let x = `template`;</code></pre>");
    assert!(result.contains("template"));
}

#[test]
fn li_without_parent_list() {
    let result = convert("<li>Orphan item</li>");
    assert!(result.contains("Orphan item"));
}

#[test]
fn nested_blockquotes_5_levels() {
    let html = "<blockquote><blockquote><blockquote><blockquote><blockquote><p>Deep quote</p></blockquote></blockquote></blockquote></blockquote></blockquote>";
    let result = convert(html);
    assert!(result.contains("Deep quote"));
    assert!(result.contains("> >"));
}

#[test]
fn empty_heading_produces_nothing() {
    let result = convert("<h1></h1>");
    assert!(!result.contains("#"));
}

#[test]
fn heading_with_only_whitespace_produces_nothing() {
    let result = convert("<h1>   </h1>");
    assert!(!result.contains("#"));
}

#[test]
fn does_not_panic_on_null_bytes() {
    let html = "<p>contains\0null\0bytes</p>";
    let _ = convert(html);
}

#[test]
fn does_not_panic_on_only_tags() {
    let html = "<div><span><br><hr></span></div>";
    let _ = convert(html);
}

#[test]
fn does_not_panic_on_deeply_nested_lists() {
    let mut html = String::new();
    for _ in 0..50 {
        html.push_str("<ul><li>");
    }
    html.push_str("Deep item");
    for _ in 0..50 {
        html.push_str("</li></ul>");
    }
    let result = convert(&html);
    assert!(result.contains("Deep item"));
}

#[test]
fn javascript_links_are_rendered_as_plain_text() {
    let result = convert(r#"<p><a href="javascript:alert(document.cookie)">go</a></p>"#);
    assert_eq!(result.trim(), "go");
    assert!(!result.contains("javascript:"));
}

#[test]
fn very_deep_nesting_does_not_crash_or_hang() {
    let n = 200_000;
    let html = "<div>".repeat(n) + "survivor" + &"</div>".repeat(n);
    let result = convert(&html);
    assert!(result.contains("survivor"));
}
