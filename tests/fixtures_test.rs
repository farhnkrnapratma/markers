use std::fs;
use std::path::PathBuf;

use markers::{convert, convert_with_options, ConversionOptions};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn load_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

#[test]
fn test_blog_post_headings() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    assert!(markdown.contains("# Getting Started with Rust"));

    assert!(markdown.contains("## Why Rust?"));
    assert!(markdown.contains("## Installation"));
    assert!(markdown.contains("## Your First Program"));
    assert!(markdown.contains("## Conclusion"));
}

#[test]
fn test_blog_post_code_blocks() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    assert!(markdown.contains("```rust") || markdown.contains("```"));
    assert!(markdown.contains("fn main()"));
    assert!(markdown.contains("println!"));

    assert!(markdown.contains("Hello, World!"));
}

#[test]
fn test_blog_post_lists() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    assert!(markdown.contains("- ") || markdown.contains("* "));
}

#[test]
fn test_blog_post_blockquote() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    assert!(markdown.contains(">"));
}

#[test]
fn test_blog_post_links() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    assert!(markdown.contains("[official Rust Book](https://doc.rust-lang.org/book/)"));
}

#[test]
fn test_documentation_tables() {
    let html = load_fixture("documentation.html");
    let markdown = convert(&html);

    assert!(markdown.contains("|"));
    assert!(markdown.contains("Name"));
    assert!(markdown.contains("Type"));
    assert!(markdown.contains("Required"));
}

#[test]
fn test_documentation_code_in_text() {
    let html = load_fixture("documentation.html");
    let markdown = convert(&html);

    assert!(markdown.contains("`convert(html, options?)`"));
    assert!(markdown.contains("`string`"));
}

#[test]
fn test_documentation_definition_lists() {
    let html = load_fixture("documentation.html");
    let markdown = convert(&html);

    assert!(markdown.contains("headingStyle"));
    assert!(markdown.contains("linkStyle"));
    assert!(markdown.contains("excludeSelectors"));
}

#[test]
fn test_documentation_details_element() {
    let html = load_fixture("documentation.html");
    let markdown = convert(&html);

    assert!(markdown.contains("Block Elements"));
    assert!(markdown.contains("Inline Elements"));
    assert!(markdown.contains("Headings"));
}

#[test]
fn test_tables_basic() {
    let html = load_fixture("tables.html");
    let markdown = convert(&html);

    assert!(markdown.contains("Feature"));
    assert!(markdown.contains("Free"));
    assert!(markdown.contains("Pro"));
    assert!(markdown.contains("Enterprise"));

    assert!(markdown.contains("API Requests"));
    assert!(markdown.contains("Unlimited"));
}

#[test]
fn test_tables_code_in_cells() {
    let html = load_fixture("tables.html");
    let markdown = convert(&html);

    assert!(markdown.contains("`200`") || markdown.contains("200"));
    assert!(markdown.contains("OK"));
    assert!(markdown.contains("Request succeeded"));
}

#[test]
fn test_tables_separator() {
    let html = load_fixture("tables.html");
    let markdown = convert(&html);

    assert!(markdown.contains("|---") || markdown.contains("| ---"));
}

#[test]
fn test_code_heavy_multiple_languages() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    assert!(markdown.contains("```bash") || markdown.contains("```"));
    assert!(markdown.contains("```rust") || markdown.contains("```"));
    assert!(markdown.contains("```toml") || markdown.contains("```"));
}

#[test]
fn test_code_heavy_preserves_content() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    assert!(markdown.contains("fn main()"));
    assert!(markdown.contains("println!"));
    assert!(markdown.contains("cargo new"));
}

#[test]
fn test_code_heavy_inline_code() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    assert!(markdown.contains("`println!`"));
    assert!(markdown.contains("`cargo new`"));
}

#[test]
fn test_code_heavy_special_chars() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    assert!(markdown.contains("->") || markdown.contains("-&gt;"));
    assert!(markdown.contains("&str") || markdown.contains("&amp;str"));
}

#[test]
fn test_malformed_no_panic() {
    let html = load_fixture("malformed.html");

    let _markdown = convert(&html);
}

#[test]
fn test_malformed_recovers_content() {
    let html = load_fixture("malformed.html");
    let markdown = convert(&html);

    assert!(markdown.contains("Welcome"));

    assert!(markdown.contains("properly formatted") || markdown.contains("Valid Content"));
}

#[test]
fn test_malformed_handles_unclosed_tags() {
    let html = load_fixture("malformed.html");
    let markdown = convert(&html);

    assert!(markdown.contains("First item"));
    assert!(markdown.contains("Second item"));
    assert!(markdown.contains("Third item"));
}

#[test]
fn test_malformed_strips_script_with_exclude() {
    let html = load_fixture("malformed.html");
    let options = ConversionOptions::new().exclude_selectors(vec!["script".to_string()]);
    let markdown = convert_with_options(&html, &options);

    assert!(!markdown.contains("alert"));
    assert!(!markdown.contains("XSS"));
}

#[test]
fn test_malformed_strips_style_with_exclude() {
    let html = load_fixture("malformed.html");
    let options = ConversionOptions::new().exclude_selectors(vec!["style".to_string()]);
    let markdown = convert_with_options(&html, &options);

    assert!(!markdown.contains("background: red"));
}

#[test]
fn test_malformed_handles_entities() {
    let html = load_fixture("malformed.html");
    let markdown = convert(&html);

    assert!(markdown.contains("©") || markdown.contains("&copy;"));
}

#[test]
fn test_malformed_valid_list_at_end() {
    let html = load_fixture("malformed.html");
    let markdown = convert(&html);

    let lower = markdown.to_lowercase();
    assert!(lower.contains("item one") || markdown.contains("Item one"));
    assert!(lower.contains("item two") || markdown.contains("Item two"));
    assert!(lower.contains("item three") || markdown.contains("Item three"));
}

#[test]
fn test_exclude_nav() {
    let html = load_fixture("documentation.html");
    let options = ConversionOptions::new().exclude_selectors(vec![".docs-nav".to_string()]);
    let markdown = convert_with_options(&html, &options);

    assert!(!markdown.contains("[Home](/)"));
}

#[test]
fn test_exclude_footer() {
    let html = load_fixture("documentation.html");
    let options = ConversionOptions::new().exclude_selectors(vec!["footer".to_string()]);
    let markdown = convert_with_options(&html, &options);

    assert!(!markdown.contains("markers documentation"));
}

#[test]
fn test_empty_string() {
    let markdown = convert("");
    assert!(markdown.is_empty() || markdown.trim().is_empty());
}

#[test]
fn test_whitespace_only() {
    let markdown = convert("   \n\t\n   ");
    assert!(markdown.trim().is_empty());
}

#[test]
fn test_plain_text() {
    let markdown = convert("Just plain text without any HTML");
    assert!(markdown.contains("Just plain text"));
}

#[test]
fn test_deeply_nested() {
    let html = "<div><div><div><div><div><p>Deep content</p></div></div></div></div></div>";
    let markdown = convert(html);
    assert!(markdown.contains("Deep content"));
}

#[test]
fn test_long_document() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    assert!(markdown.contains("Getting Started with Rust"));
    assert!(markdown.contains("Error Handling"));
}
