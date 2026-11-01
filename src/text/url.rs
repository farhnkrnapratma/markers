pub fn has_unsafe_scheme(url: &str) -> bool {
    let head: String = url
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .skip_while(|c| c.is_ascii_control() || *c == ' ')
        .take(11)
        .collect::<String>()
        .to_ascii_lowercase();
    head.starts_with("javascript:") || head.starts_with("vbscript:") || head.starts_with("data:")
}

pub fn resolve_against_base(base: &str, relative: &str) -> String {
    if relative.starts_with("http://")
        || relative.starts_with("https://")
        || relative.starts_with("//")
        || relative.starts_with("mailto:")
        || relative.starts_with("tel:")
        || relative.starts_with("data:")
    {
        return relative.to_string();
    }

    if relative.starts_with('/') {
        if let Some(protocol_end) = base.find("://") {
            let after_protocol = &base[protocol_end + 3..];
            let origin_end = after_protocol
                .find('/')
                .map(|j| protocol_end + 3 + j)
                .unwrap_or(base.len());
            format!("{}{}", &base[..origin_end], relative)
        } else {
            format!("{}{}", base, relative)
        }
    } else if relative.starts_with('#') || relative.starts_with('?') {
        let base = base.trim_end_matches('/');
        format!("{}{}", base, relative)
    } else {
        let origin_only = base
            .find("://")
            .is_some_and(|i| !base[i + 3..].contains('/'));
        if base.ends_with('/') || origin_only {
            let sep = if base.ends_with('/') { "" } else { "/" };
            format!("{}{}{}", base, sep, relative)
        } else if let Some(last_slash) = base.rfind('/') {
            format!("{}/{}", &base[..last_slash], relative)
        } else {
            format!("{}/{}", base, relative)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_against_base() {
        assert_eq!(
            resolve_against_base("https://example.com", "https://other.com/page"),
            "https://other.com/page"
        );

        assert_eq!(
            resolve_against_base("https://example.com/dir/page", "/other"),
            "https://example.com/other"
        );

        assert_eq!(
            resolve_against_base("https://example.com/dir/page", "other"),
            "https://example.com/dir/other"
        );

        assert_eq!(
            resolve_against_base("https://example.com/page", "#section"),
            "https://example.com/page#section"
        );
    }

    #[test]
    fn test_has_unsafe_scheme_detects_dangerous_schemes() {
        assert!(has_unsafe_scheme("javascript:alert(1)"));
        assert!(has_unsafe_scheme("JaVaScRiPt:alert(1)"));
        assert!(has_unsafe_scheme("vbscript:msgbox(1)"));
        assert!(has_unsafe_scheme("data:text/html;base64,AAAA"));
        assert!(has_unsafe_scheme("  javascript:alert(1)"));
        assert!(has_unsafe_scheme("\u{1}javascript:alert(1)"));
        assert!(has_unsafe_scheme("java\tscript:alert(1)"));
        assert!(has_unsafe_scheme("java\nscript:alert(1)"));
    }

    #[test]
    fn test_has_unsafe_scheme_allows_normal_urls() {
        assert!(!has_unsafe_scheme("https://example.com"));
        assert!(!has_unsafe_scheme("http://example.com/javascript:"));
        assert!(!has_unsafe_scheme("mailto:a@b.c"));
        assert!(!has_unsafe_scheme("/relative/javascript:alert"));
        assert!(!has_unsafe_scheme("#frag"));
        assert!(!has_unsafe_scheme("javascript"));
        assert!(!has_unsafe_scheme(""));
    }

    #[test]
    fn test_resolve_against_base_keeps_host_when_base_has_no_path() {
        assert_eq!(
            resolve_against_base("https://example.com", "a.png"),
            "https://example.com/a.png"
        );
        assert_eq!(
            resolve_against_base("https://example.com", "img/a b.png"),
            "https://example.com/img/a b.png"
        );
    }
}
