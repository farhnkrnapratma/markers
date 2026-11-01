pub const MAX_INPUT_NESTING: usize = 5_000;

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

const OPTIONAL_END: &[&str] = &[
    "html", "head", "body", "li", "p", "dt", "dd", "tr", "td", "th", "tbody", "thead", "tfoot",
    "option", "optgroup", "colgroup", "rp", "rt",
];

fn tag_end(bytes: &[u8], mut i: usize) -> usize {
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) if b == q => quote = None,
            Some(_) => {}
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'>' => return i + 1,
                _ => {}
            },
        }
        i += 1;
    }
    bytes.len()
}

fn markup_end(bytes: &[u8], i: usize) -> usize {
    if bytes[i..].starts_with(b"<!--") {
        return match find(bytes, i + 4, b"-->") {
            Some(j) => j + 3,
            None => bytes.len(),
        };
    }
    tag_end(bytes, i + 1)
}

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

fn tag_name(bytes: &[u8], start: usize) -> String {
    let end = bytes[start..]
        .iter()
        .position(|b| !(b.is_ascii_alphanumeric() || *b == b'-' || *b == b':'))
        .map_or(bytes.len(), |p| start + p);
    String::from_utf8_lossy(&bytes[start..end]).to_ascii_lowercase()
}

pub fn exceeds_nesting_limit(html: &str, limit: usize) -> bool {
    let bytes = html.as_bytes();
    let mut depth: usize = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        let next = bytes.get(i + 1).copied();
        match next {
            Some(b'!') | Some(b'?') => {
                i = markup_end(bytes, i);
            }
            Some(b'/') => {
                let name = tag_name(bytes, i + 2);
                let end = tag_end(bytes, i + 2);
                if !name.is_empty()
                    && !VOID.contains(&name.as_str())
                    && !OPTIONAL_END.contains(&name.as_str())
                {
                    depth = depth.saturating_sub(1);
                }
                i = end;
            }
            Some(c) if c.is_ascii_alphabetic() => {
                let name = tag_name(bytes, i + 1);
                let end = tag_end(bytes, i + 1);
                let self_closing = end >= 2 && bytes[end - 2] == b'/';
                if !self_closing
                    && !VOID.contains(&name.as_str())
                    && !OPTIONAL_END.contains(&name.as_str())
                {
                    depth += 1;
                    if depth > limit {
                        return true;
                    }
                }
                i = end;
            }
            _ => i += 1,
        }
    }
    false
}

pub fn strip_markup(html: &str) -> String {
    let bytes = html.as_bytes();
    let mut out = String::with_capacity(html.len() / 2);
    let mut text_start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        let next = bytes.get(i + 1).copied();
        let is_markup = matches!(next, Some(b'!') | Some(b'?') | Some(b'/'))
            || next.is_some_and(|c| c.is_ascii_alphabetic());
        if !is_markup {
            i += 1;
            continue;
        }
        out.push_str(&html[text_start..i]);
        i = markup_end(bytes, i);
        text_start = i;
    }
    out.push_str(&html[text_start..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested(tag: &str, n: usize) -> String {
        format!("<{tag}>").repeat(n) + "x" + &format!("</{tag}>").repeat(n)
    }

    #[test]
    fn test_shallow_document_is_within_limit() {
        assert!(!exceeds_nesting_limit(
            "<div><p>a</p><ul><li>b</li></ul></div>",
            10
        ));
    }

    #[test]
    fn test_deep_nesting_exceeds_limit() {
        assert!(exceeds_nesting_limit(&nested("div", 11), 10));
        assert!(!exceeds_nesting_limit(&nested("div", 10), 10));
    }

    #[test]
    fn test_unclosed_optional_end_tags_do_not_count() {
        let html = "<ul>".to_string() + &"<li>x".repeat(1000) + "</ul>";
        assert!(!exceeds_nesting_limit(&html, 10));
        let html = "<p>x".repeat(1000);
        assert!(!exceeds_nesting_limit(&html, 10));
    }

    #[test]
    fn test_void_and_self_closing_tags_do_not_count() {
        assert!(!exceeds_nesting_limit(
            &"<br><img src=a><hr/><div/>".repeat(100),
            3
        ));
    }

    #[test]
    fn test_closing_tags_decrease_depth() {
        assert!(!exceeds_nesting_limit(&"<div></div>".repeat(1000), 1));
    }

    #[test]
    fn test_comments_and_quoted_angle_brackets_are_ignored() {
        assert!(!exceeds_nesting_limit(
            &"<!-- <div><div><div> -->".repeat(100),
            2
        ));
        assert!(!exceeds_nesting_limit(
            &r#"<a title="<div><div><div>">x</a>"#.repeat(100),
            2
        ));
    }

    #[test]
    fn test_tag_names_are_case_insensitive() {
        assert!(exceeds_nesting_limit(&nested("DIV", 11), 10));
        assert!(!exceeds_nesting_limit(&"<LI>x".repeat(100), 3));
    }

    #[test]
    fn test_malformed_input_terminates() {
        assert!(!exceeds_nesting_limit("<div", 5));
        assert!(!exceeds_nesting_limit("<", 5));
        assert!(!exceeds_nesting_limit("<!-- never closed", 5));
        assert!(!exceeds_nesting_limit("</", 5));
        assert!(!exceeds_nesting_limit("a < b > c", 5));
    }

    #[test]
    fn test_strip_markup_keeps_text() {
        assert_eq!(strip_markup("<div><p>a <b>b</b></p>c</div>"), "a bc");
        assert_eq!(strip_markup("a<!-- hidden -->b"), "ab");
        assert_eq!(strip_markup(r#"<a title="x>y">z</a>"#), "z");
        assert_eq!(strip_markup("1 < 2 and 3 > 2"), "1 < 2 and 3 > 2");
        assert_eq!(
            strip_markup("caf\u{e9} <b>\u{4e2d}\u{6587}</b>"),
            "caf\u{e9} \u{4e2d}\u{6587}"
        );
    }

    #[test]
    fn test_strip_markup_unterminated_tag_is_dropped() {
        assert_eq!(strip_markup("text <div class=\"a"), "text ");
    }
}
