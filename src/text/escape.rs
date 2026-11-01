pub fn escape_link_title(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' | '\\' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

pub fn escape_inline_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' | '[' | ']' => {
                result.push('\\');
                result.push(c);
            }

            '<' if chars
                .peek()
                .is_none_or(|n| n.is_ascii_alphabetic() || matches!(n, '/' | '!' | '?')) =>
            {
                result.push_str("\\<");
            }
            _ => result.push(c),
        }
    }
    result
}

pub fn escape_image_alt(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '[' | ']' | '\\') {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

pub fn escape_link_destination(url: &str) -> String {
    let mut result = String::with_capacity(url.len());
    for c in url.chars() {
        match c {
            '(' => result.push_str("%28"),
            ')' => result.push_str("%29"),
            ' ' => result.push_str("%20"),
            '<' => result.push_str("%3C"),
            '>' => result.push_str("%3E"),
            '\\' => result.push_str("%5C"),
            c if c.is_ascii_control() => result.push_str(&format!("%{:02X}", c as u32)),
            _ => result.push(c),
        }
    }
    result
}

pub fn backtick_delimiter_len(code: &str) -> usize {
    let mut max_run = 0;
    let mut current_run = 0;

    for c in code.chars() {
        if c == '`' {
            current_run += 1;
            max_run = max_run.max(current_run);
        } else {
            current_run = 0;
        }
    }

    if max_run == 0 {
        1
    } else {
        max_run + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_link_title() {
        assert_eq!(escape_link_title(r#"Hello "World""#), r#"Hello \"World\""#);
        assert_eq!(escape_link_title(r"path\to\file"), r"path\\to\\file");
    }

    #[test]
    fn test_escape_link_destination() {
        assert_eq!(
            escape_link_destination("url (with parens)"),
            "url%20%28with%20parens%29"
        );
    }

    #[test]
    fn test_backtick_delimiter_len() {
        assert_eq!(backtick_delimiter_len("no backticks"), 1);
        assert_eq!(backtick_delimiter_len("one ` backtick"), 2);
        assert_eq!(backtick_delimiter_len("two `` backticks"), 3);
        assert_eq!(backtick_delimiter_len("```"), 4);
    }

    #[test]
    fn test_escape_link_destination_encodes_angle_brackets_backslash_and_controls() {
        assert_eq!(escape_link_destination("a<b>c"), "a%3Cb%3Ec");
        assert_eq!(escape_link_destination("a\\b"), "a%5Cb");
        assert_eq!(escape_link_destination("a\nb\rc\td"), "a%0Ab%0Dc%09d");
        assert_eq!(escape_link_destination("a\u{7f}b"), "a%7Fb");
        assert_eq!(
            escape_link_destination("https://example.com/\u{4e2d}"),
            "https://example.com/\u{4e2d}"
        );
    }

    #[test]
    fn test_escape_image_alt() {
        assert_eq!(escape_image_alt("plain alt"), "plain alt");
        assert_eq!(escape_image_alt("a](x)[b"), r"a\](x)\[b");
        assert_eq!(escape_image_alt(r"back\slash"), r"back\\slash");
    }

    #[test]
    fn test_escape_inline_text() {
        assert_eq!(
            escape_inline_text("plain 1 < 2 > 0 *a* _b_ # c"),
            "plain 1 < 2 > 0 *a* _b_ # c"
        );
        assert_eq!(escape_inline_text(r"a\b [c] d"), r"a\\b \[c\] d");
        assert_eq!(
            escape_inline_text("<a> </a> <!-- <?x"),
            r"\<a> \</a> \<!-- \<?x"
        );
        assert_eq!(escape_inline_text("ends with <"), r"ends with \<");
    }
}
