use once_cell::sync::Lazy;
use regex::Regex;

static ALL_WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub fn collapse_whitespace(text: &str) -> String {
    ALL_WS_RE.replace_all(text, " ").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapse_whitespace() {
        assert_eq!(collapse_whitespace("hello   world"), "hello world");
        assert_eq!(collapse_whitespace("line1\n\nline2"), "line1 line2");
        assert_eq!(collapse_whitespace("  spaced  out  "), " spaced out ");
    }
}
