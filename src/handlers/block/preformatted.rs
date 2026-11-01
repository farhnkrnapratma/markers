use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};

static BACKTICK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`+").unwrap());
static TILDE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"~+").unwrap());

pub struct PreformattedHandler;

impl TagHandler for PreformattedHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["pre"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let lang = detect_language(&element)
            .filter(|lang| is_safe_info_string(lang))
            .unwrap_or_default();

        let code = collect_code_text(&element);
        let code = code.trim_end_matches('\n');

        if code.is_empty() {
            return String::new();
        }

        let fence = calculate_fence(code, cx.options.code_fence);

        format!("\n\n{}{}\n{}\n{}\n\n", fence, lang, code, fence)
    }
}

fn is_safe_info_string(lang: &str) -> bool {
    lang.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '+' | '.' | '#'))
}

fn detect_language(pre: &ElementRef) -> Option<String> {
    if let Some(class) = pre.value().attr("class") {
        if let Some(lang) = extract_language_from_class(class) {
            return Some(lang);
        }
    }

    for child in pre.children() {
        if let Some(element) = ElementRef::wrap(child) {
            if element.value().name() == "code" {
                if let Some(class) = element.value().attr("class") {
                    if let Some(lang) = extract_language_from_class(class) {
                        return Some(lang);
                    }
                }
            }
        }
    }

    None
}

static KNOWN_LANGUAGES: &[&str] = &[
    "bash",
    "c",
    "cpp",
    "csharp",
    "css",
    "dart",
    "diff",
    "go",
    "graphql",
    "html",
    "java",
    "javascript",
    "js",
    "json",
    "kotlin",
    "lua",
    "makefile",
    "markdown",
    "objectivec",
    "perl",
    "php",
    "plaintext",
    "python",
    "r",
    "ruby",
    "rust",
    "scala",
    "shell",
    "sql",
    "swift",
    "typescript",
    "ts",
    "xml",
    "yaml",
    "yml",
];

fn extract_language_from_class(class: &str) -> Option<String> {
    for part in class.split_whitespace() {
        if let Some(lang) = part.strip_prefix("language-") {
            return Some(lang.to_string());
        }
        if let Some(lang) = part.strip_prefix("lang-") {
            return Some(lang.to_string());
        }
        if let Some(lang) = part.strip_prefix("highlight-") {
            return Some(lang.to_string());
        }
        if let Some(lang) = part.strip_prefix("hljs-") {
            if ![
                "keyword", "string", "number", "comment", "function", "class", "built_in",
            ]
            .contains(&lang)
            {
                return Some(lang.to_string());
            }
        }
    }

    for part in class.split_whitespace() {
        let lower = part.to_lowercase();
        if KNOWN_LANGUAGES.contains(&lower.as_str()) {
            return Some(lower);
        }
    }

    None
}

fn calculate_fence(code: &str, preferred: char) -> String {
    let re = match preferred {
        '~' => &*TILDE_RE,
        _ => &*BACKTICK_RE,
    };

    let max_run = re
        .find_iter(code)
        .map(|m| m.as_str().len())
        .max()
        .unwrap_or(0);

    let fence_len = std::cmp::max(3, max_run + 1);
    std::iter::repeat_n(preferred, fence_len).collect()
}

fn collect_code_text(pre: &ElementRef) -> String {
    let mut text = String::new();

    fn collect_recursive(node: ego_tree::NodeRef<scraper::Node>, text: &mut String, skip: bool) {
        if let Some(element) = ElementRef::wrap(node) {
            if let Some(class) = element.value().attr("class") {
                if class.contains("gutter")
                    || class.contains("line-number")
                    || class.contains("line-numbers")
                    || class.contains("lineno")
                    || class.contains("linenumber")
                {
                    return;
                }
            }
        }

        match node.value() {
            scraper::Node::Text(t) if !skip => {
                text.push_str(t);
            }
            scraper::Node::Element(_) => {
                for child in node.children() {
                    collect_recursive(child, text, false);
                }
            }
            _ => {}
        }
    }

    for child in pre.children() {
        collect_recursive(child, &mut text, false);
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render_with;
    use crate::options::ConversionOptions;

    fn convert_test(html: &str, options: &ConversionOptions) -> String {
        render_with(&PreformattedHandler, html, options)
    }

    #[test]
    fn test_simple_pre() {
        let result = convert_test("<pre>code here</pre>", &ConversionOptions::default());
        assert!(result.contains("```"));
        assert!(result.contains("code here"));
    }

    #[test]
    fn test_pre_with_language() {
        let result = convert_test(
            r#"<pre><code class="language-rust">fn main() {}</code></pre>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_pre_with_backticks() {
        let result = convert_test("<pre>use ``` here</pre>", &ConversionOptions::default());
        assert!(result.contains("````"));
    }

    #[test]
    fn test_tilde_fence() {
        let options = ConversionOptions::new().code_fence('~');
        let result = convert_test("<pre>code</pre>", &options);
        assert!(result.contains("~~~"));
    }

    #[test]
    fn test_empty_pre() {
        let result = convert_test("<pre></pre>", &ConversionOptions::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_highlight_prefix() {
        let result = convert_test(
            r#"<pre><code class="highlight-python">print("hello")</code></pre>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("```python"));
    }

    #[test]
    fn test_bare_language_fallback() {
        let result = convert_test(
            r#"<pre><code class="javascript">console.log()</code></pre>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("```javascript"));
    }

    #[test]
    fn test_lang_prefix() {
        let result = convert_test(
            r#"<pre><code class="lang-go">func main()</code></pre>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("```go"));
    }

    #[test]
    fn test_hljs_language() {
        let result = convert_test(
            r#"<pre><code class="hljs-typescript">const x: string = "test";</code></pre>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("```typescript"));
    }

    #[test]
    fn test_hljs_token_class_ignored() {
        let result = convert_test(
            r#"<pre><code class="hljs-keyword">const</code></pre>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("```\n"));
    }

    #[test]
    fn test_multiple_backticks_in_code() {
        let result = convert_test(
            "<pre>Here is some code:\n```\nmore code\n```</pre>",
            &ConversionOptions::default(),
        );
        assert!(result.contains("````"));
    }

    #[test]
    fn test_gutter_classes_skipped() {
        let result = convert_test(
            r#"<pre><code><span class="line-numbers">1</span>first line
<span class="lineno">2</span>second line</code></pre>"#,
            &ConversionOptions::default(),
        );
        assert!(!result.contains("1first"));
        assert!(!result.contains("2second"));
        assert!(result.contains("first line"));
        assert!(result.contains("second line"));
    }

    #[test]
    fn test_pre_on_element_class() {
        let result = convert_test(
            r#"<pre class="language-sql">SELECT * FROM users;</pre>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("```sql"));
    }

    #[test]
    fn test_multiline_code() {
        let result = convert_test(
            r#"<pre><code>line 1
line 2
line 3</code></pre>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("line 1\nline 2\nline 3"));
    }

    #[test]
    fn test_whitespace_preserved() {
        let result = convert_test(
            "<pre>    indented\n        more</pre>",
            &ConversionOptions::default(),
        );
        assert!(result.contains("    indented"));
        assert!(result.contains("        more"));
    }

    #[test]
    fn test_language_with_a_fence_character_is_dropped() {
        for class in ["language-js`x", "language-a~b", "language-~~~"] {
            let html = format!(r#"<pre><code class="{class}">a</code></pre>"#);
            assert_eq!(
                convert_test(&html, &ConversionOptions::default()),
                "\n\n```\na\n```\n\n",
                "{class}"
            );
        }
    }

    #[test]
    fn test_language_names_with_symbols_are_kept() {
        for (class, lang) in [
            ("language-c++", "c++"),
            ("language-c#", "c#"),
            ("language-objective-c", "objective-c"),
            ("lang-vue.js", "vue.js"),
            ("language-x y`z", "x"),
        ] {
            let html = format!(r#"<pre><code class="{class}">a</code></pre>"#);
            assert_eq!(
                convert_test(&html, &ConversionOptions::default()),
                format!("\n\n```{lang}\na\n```\n\n"),
                "{class}"
            );
        }
    }
}
