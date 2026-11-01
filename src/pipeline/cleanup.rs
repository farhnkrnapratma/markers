use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap;

use crate::options::{ConversionOptions, LinkStyle};

static BLANK_LINE_RUN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());

static INLINE_LINK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(^|[^!])\[([^\]]+)\]\(([^)\s]+)(?:\s+"([^"]*)")?\)"#).unwrap());

pub fn finalize_markdown(markdown: String, options: &ConversionOptions) -> String {
    let mut result = markdown;

    result = escape_newlines_in_link_text(&result);

    if matches!(options.link_style, LinkStyle::Referenced) {
        result = inline_links_to_references(&result);
    }

    result = trim_line_ends(&result);

    result = BLANK_LINE_RUN_RE.replace_all(&result, "\n\n").into_owned();

    result.trim().to_string()
}

fn inline_links_to_references(markdown: &str) -> String {
    let mut url_to_ref: FxHashMap<String, usize> = FxHashMap::default();
    let mut references: Vec<(usize, String, Option<String>)> = Vec::new();
    let mut ref_counter = 0;

    let result = INLINE_LINK_RE.replace_all(markdown, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let text = &caps[2];
        let url = &caps[3];
        let title = caps.get(4).map(|m| m.as_str().to_string());

        let ref_num = if let Some(&existing_ref) = url_to_ref.get(url) {
            existing_ref
        } else {
            ref_counter += 1;
            url_to_ref.insert(url.to_string(), ref_counter);
            references.push((ref_counter, url.to_string(), title));
            ref_counter
        };

        format!("{}[{}][{}]", prefix, text, ref_num)
    });

    if references.is_empty() {
        return markdown.to_string();
    }

    let mut output = result.into_owned();
    output.push_str("\n\n");

    for (num, url, title) in references {
        match title {
            Some(t) => output.push_str(&format!("[{}]: {} \"{}\"\n", num, url, t)),
            None => output.push_str(&format!("[{}]: {}\n", num, url)),
        }
    }

    output
}

fn escape_newlines_in_link_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut bracket_depth: i32 = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next == '[' || next == ']' {
                result.push(c);
                result.push(next);
                i += 2;
                continue;
            }
        }

        match c {
            '[' => {
                bracket_depth += 1;
                result.push(c);
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                result.push(c);
            }
            '\n' if bracket_depth > 0 => {
                result.push_str("\\n");
            }
            _ => result.push(c),
        }
        i += 1;
    }

    result
}

fn trim_line_ends(text: &str) -> String {
    text.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_newlines_in_link_text() {
        let input = "[text\nwith newline](url)";
        let result = escape_newlines_in_link_text(input);
        assert_eq!(result, "[text\\nwith newline](url)");
    }

    #[test]
    fn test_escape_newlines_in_link_text_escaped_bracket() {
        let input = r"text with \[ escaped bracket";
        let result = escape_newlines_in_link_text(input);
        assert_eq!(result, r"text with \[ escaped bracket");
    }

    #[test]
    fn test_escape_newlines_in_link_text_no_change() {
        let input = "normal text\nwith newline outside brackets";
        let result = escape_newlines_in_link_text(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_inline_links_to_references() {
        let input = "Check [this](https://a.com) and [that](https://b.com).";
        let result = inline_links_to_references(input);
        assert!(result.contains("[this][1]"));
        assert!(result.contains("[that][2]"));
        assert!(result.contains("[1]: https://a.com"));
        assert!(result.contains("[2]: https://b.com"));
    }

    #[test]
    fn test_inline_links_to_references_dedup() {
        let input = "[a](https://x.com) and [b](https://x.com)";
        let result = inline_links_to_references(input);
        assert!(result.contains("[a][1]"));
        assert!(result.contains("[b][1]"));
        assert_eq!(result.matches("[1]:").count(), 1);
    }

    #[test]
    fn test_inline_links_to_references_with_title() {
        let input = r#"[link](https://a.com "Title")"#;
        let result = inline_links_to_references(input);
        assert!(result.contains("[link][1]"));
        assert!(result.contains(r#"[1]: https://a.com "Title""#));
    }

    #[test]
    fn test_inline_links_to_references_no_images() {
        let input = "![image](img.png) and [link](url)";
        let result = inline_links_to_references(input);
        assert!(result.contains("![image](img.png)"));
        assert!(result.contains("[link][1]"));
    }

    #[test]
    fn test_trim_line_ends() {
        let input = "line 1   \nline 2  \nline 3";
        let result = trim_line_ends(input);
        assert_eq!(result, "line 1\nline 2\nline 3");
    }

    #[test]
    fn test_collapse_newlines() {
        let input = "a\n\n\n\nb";
        let result = BLANK_LINE_RUN_RE.replace_all(input, "\n\n");
        assert_eq!(result, "a\n\nb");
    }

    #[test]
    fn test_finalize_markdown_full() {
        let input = "# Title\n\n\n\nParagraph   \n";
        let result = finalize_markdown(input.to_string(), &ConversionOptions::default());
        assert_eq!(result, "# Title\n\nParagraph");
    }

    #[test]
    fn test_finalize_markdown_collapses_whitespace_only_lines() {
        let input = "# Title\n\n \n\nParagraph";
        let result = finalize_markdown(input.to_string(), &ConversionOptions::default());
        assert_eq!(result, "# Title\n\nParagraph");
    }
}
