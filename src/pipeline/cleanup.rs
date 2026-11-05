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
    let mut output = String::with_capacity(markdown.len());
    let mut prose = String::new();
    let mut fence: Option<(u8, usize)> = None;

    for line in markdown.split_inclusive('\n') {
        let line_fence = markdown_fence(line);
        match fence {
            Some((character, length)) => {
                output.push_str(line);
                if line_fence.is_some_and(|(found, found_length, end)| {
                    found == character && found_length >= length && line[end..].trim().is_empty()
                }) {
                    fence = None;
                }
            }
            None => {
                if let Some((character, length, _)) = line_fence {
                    output.push_str(&replace_links_outside_code(
                        &prose,
                        &mut url_to_ref,
                        &mut references,
                        &mut ref_counter,
                    ));
                    prose.clear();
                    output.push_str(line);
                    fence = Some((character, length));
                } else {
                    prose.push_str(line);
                }
            }
        }
    }

    output.push_str(&replace_links_outside_code(
        &prose,
        &mut url_to_ref,
        &mut references,
        &mut ref_counter,
    ));

    if references.is_empty() {
        return output;
    }

    output.push_str("\n\n");

    for (num, url, title) in references {
        match title {
            Some(t) => output.push_str(&format!("[{}]: {} \"{}\"\n", num, url, t)),
            None => output.push_str(&format!("[{}]: {}\n", num, url)),
        }
    }

    output
}

fn markdown_fence(line: &str) -> Option<(u8, usize, usize)> {
    let indent = line.bytes().take_while(|b| *b == b' ').count();
    if indent > 3 {
        return None;
    }

    let rest = &line.as_bytes()[indent..];
    let character = *rest.first()?;
    if character != b'`' && character != b'~' {
        return None;
    }

    let length = rest.iter().take_while(|b| **b == character).count();
    (length >= 3).then_some((character, length, indent + length))
}

fn replace_links_outside_code(
    text: &str,
    url_to_ref: &mut FxHashMap<String, usize>,
    references: &mut Vec<(usize, String, Option<String>)>,
    ref_counter: &mut usize,
) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut segment_start = 0;
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'`' {
            index += 1;
            continue;
        }

        let delimiter_length = bytes[index..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        let delimiter_end = index + delimiter_length;
        let Some(code_end) = find_code_span_end(bytes, delimiter_end, delimiter_length) else {
            index = delimiter_end;
            continue;
        };

        output.push_str(&replace_links_in_prose(
            &text[segment_start..index],
            url_to_ref,
            references,
            ref_counter,
        ));
        output.push_str(&text[index..code_end]);
        index = code_end;
        segment_start = index;
    }

    output.push_str(&replace_links_in_prose(
        &text[segment_start..],
        url_to_ref,
        references,
        ref_counter,
    ));
    output
}

fn find_code_span_end(bytes: &[u8], mut index: usize, delimiter_length: usize) -> Option<usize> {
    while index < bytes.len() {
        let next = bytes[index..].iter().position(|byte| *byte == b'`')? + index;
        let run_length = bytes[next..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        if run_length == delimiter_length {
            return Some(next + run_length);
        }
        index = next + run_length;
    }
    None
}

fn replace_links_in_prose(
    text: &str,
    url_to_ref: &mut FxHashMap<String, usize>,
    references: &mut Vec<(usize, String, Option<String>)>,
    ref_counter: &mut usize,
) -> String {
    INLINE_LINK_RE
        .replace_all(text, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let link_text = &caps[2];
            let url = &caps[3];
            let title = caps.get(4).map(|m| m.as_str().to_string());

            let ref_num = if let Some(&existing_ref) = url_to_ref.get(url) {
                existing_ref
            } else {
                *ref_counter += 1;
                url_to_ref.insert(url.to_string(), *ref_counter);
                references.push((*ref_counter, url.to_string(), title));
                *ref_counter
            };

            format!("{}[{}][{}]", prefix, link_text, ref_num)
        })
        .into_owned()
}

fn escape_newlines_in_link_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut bracket_depth: i32 = 0;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(&next) = chars.peek()
            && (next == '[' || next == ']')
        {
            result.push(c);
            result.push(next);
            chars.next();
            continue;
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
    fn test_inline_links_to_references_skips_code_spans_and_fences() {
        let input = concat!(
            "`[inline](https://code.example)` and [actual](https://link.example)\n",
            "~~~rust\n[block](https://code.example)\n~~~\n",
            "````\n[fenced](https://code.example)\n````\n",
        );
        let result = inline_links_to_references(input);

        assert!(result.contains("`[inline](https://code.example)`"));
        assert!(result.contains("~~~rust\n[block](https://code.example)\n~~~"));
        assert!(result.contains("````\n[fenced](https://code.example)\n````"));
        assert!(result.contains("[actual][1]"));
        assert!(result.contains("[1]: https://link.example"));
        assert_eq!(result.matches("[1]:").count(), 1);
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

    #[test]
    fn test_finalize_markdown_preserves_backslash_hard_line_breaks() {
        let result =
            finalize_markdown("first\\\nsecond".to_string(), &ConversionOptions::default());
        assert_eq!(result, "first\\\nsecond");
    }
}
