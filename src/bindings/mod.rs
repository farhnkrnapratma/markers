#[cfg(all(feature = "napi", not(target_arch = "wasm32")))]
mod node;
#[cfg(feature = "wasm")]
mod wasm;

use crate::options::{ConversionOptions, HeadingStyle, LinkStyle};

#[derive(Debug, Default)]
pub(crate) struct RawOptions {
    pub heading_style: Option<String>,
    pub link_style: Option<String>,
    pub code_fence: Option<String>,
    pub bullet_marker: Option<String>,
    pub base_url: Option<String>,
    pub exclude_selectors: Option<Vec<String>>,
    pub include_selectors: Option<Vec<String>>,
}

impl RawOptions {
    pub(crate) fn into_options(self) -> Result<ConversionOptions, String> {
        let mut options = ConversionOptions::new();

        if let Some(style) = self.heading_style {
            options = options.heading_style(style.parse::<HeadingStyle>()?);
        }
        if let Some(style) = self.link_style {
            options = options.link_style(style.parse::<LinkStyle>()?);
        }
        if let Some(fence) = self.code_fence {
            options = options.code_fence(parse_code_fence(&fence)?);
        }
        if let Some(marker) = self.bullet_marker {
            options = options.bullet_marker(parse_bullet_marker(&marker)?);
        }
        if let Some(url) = self.base_url {
            options = options.base_url(Some(url));
        }
        if let Some(selectors) = self.exclude_selectors {
            options = options.exclude_selectors(selectors);
        }
        if let Some(selectors) = self.include_selectors {
            options = options.include_selectors(selectors);
        }

        Ok(options)
    }
}

fn parse_code_fence(s: &str) -> Result<char, String> {
    match s {
        "`" => Ok('`'),
        "~" => Ok('~'),
        _ => Err(format!("invalid codeFence {s:?}: expected \"`\" or \"~\"")),
    }
}

fn parse_bullet_marker(s: &str) -> Result<char, String> {
    match s {
        "-" => Ok('-'),
        "*" => Ok('*'),
        "+" => Ok('+'),
        _ => Err(format!(
            "invalid bulletMarker {s:?}: expected \"-\", \"*\" or \"+\""
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_raw_options_give_defaults() {
        let options = RawOptions::default().into_options().unwrap();
        assert_eq!(options.heading_style, HeadingStyle::Atx);
        assert_eq!(options.bullet_marker, '-');
        assert!(options.base_url.is_none());
    }

    #[test]
    fn test_every_field_is_applied() {
        let raw = RawOptions {
            heading_style: Some("SETEXT".into()),
            link_style: Some("reference".into()),
            code_fence: Some("~".into()),
            bullet_marker: Some("+".into()),
            base_url: Some("https://example.com".into()),
            exclude_selectors: Some(vec!["nav".into()]),
            include_selectors: Some(vec![".keep".into()]),
        };
        let options = raw.into_options().unwrap();
        assert_eq!(options.heading_style, HeadingStyle::Setext);
        assert_eq!(options.link_style, LinkStyle::Referenced);
        assert_eq!(options.code_fence, '~');
        assert_eq!(options.bullet_marker, '+');
        assert_eq!(options.base_url.as_deref(), Some("https://example.com"));
        assert_eq!(options.exclude_selectors, ["nav"]);
        assert_eq!(options.include_selectors, [".keep"]);
    }

    #[test]
    fn test_the_first_invalid_field_is_reported() {
        let raw = RawOptions {
            heading_style: Some("fancy".into()),
            bullet_marker: Some("#".into()),
            ..RawOptions::default()
        };
        let message = raw.into_options().unwrap_err();
        assert!(message.contains("headingStyle"), "{message}");
    }

    #[test]
    fn test_parse_fence_and_bullet() {
        assert_eq!(parse_code_fence("`"), Ok('`'));
        assert_eq!(parse_code_fence("~"), Ok('~'));
        assert!(parse_code_fence("``").is_err());
        assert!(parse_code_fence("").is_err());
        assert_eq!(parse_bullet_marker("-"), Ok('-'));
        assert_eq!(parse_bullet_marker("*"), Ok('*'));
        assert_eq!(parse_bullet_marker("+"), Ok('+'));
        assert!(parse_bullet_marker("o").is_err());
    }
}
