#[derive(Debug, Clone)]
pub struct ConversionOptions {
    pub exclude_selectors: Vec<String>,

    pub include_selectors: Vec<String>,

    pub heading_style: HeadingStyle,

    pub code_fence: char,

    pub link_style: LinkStyle,

    pub bullet_marker: char,

    pub base_url: Option<String>,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            exclude_selectors: vec![],
            include_selectors: vec![],
            heading_style: HeadingStyle::Atx,
            code_fence: '`',
            link_style: LinkStyle::Inline,
            bullet_marker: '-',
            base_url: None,
        }
    }
}

impl ConversionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exclude_selectors(mut self, selectors: Vec<String>) -> Self {
        self.exclude_selectors = selectors;
        self
    }

    pub fn include_selectors(mut self, selectors: Vec<String>) -> Self {
        self.include_selectors = selectors;
        self
    }

    pub fn heading_style(mut self, style: HeadingStyle) -> Self {
        self.heading_style = style;
        self
    }

    pub fn code_fence(mut self, fence: char) -> Self {
        self.code_fence = fence;
        self
    }

    pub fn link_style(mut self, style: LinkStyle) -> Self {
        self.link_style = style;
        self
    }

    pub fn bullet_marker(mut self, marker: char) -> Self {
        self.bullet_marker = marker;
        self
    }

    pub fn base_url(mut self, url: Option<String>) -> Self {
        self.base_url = url;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HeadingStyle {
    #[default]
    Atx,
    Setext,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LinkStyle {
    #[default]
    Inline,
    Referenced,
}

impl std::str::FromStr for HeadingStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "atx" => Ok(Self::Atx),
            "setext" => Ok(Self::Setext),
            _ => Err(format!(
                "invalid headingStyle {s:?}: expected \"atx\" or \"setext\""
            )),
        }
    }
}

impl std::str::FromStr for LinkStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "inline" => Ok(Self::Inline),
            "referenced" | "reference" => Ok(Self::Referenced),
            _ => Err(format!(
                "invalid linkStyle {s:?}: expected \"inline\" or \"referenced\""
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = ConversionOptions::default();
        assert!(opts.exclude_selectors.is_empty());
        assert!(opts.include_selectors.is_empty());
        assert_eq!(opts.heading_style, HeadingStyle::Atx);
        assert_eq!(opts.code_fence, '`');
        assert_eq!(opts.link_style, LinkStyle::Inline);
        assert_eq!(opts.bullet_marker, '-');
        assert!(opts.base_url.is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let opts = ConversionOptions::new()
            .heading_style(HeadingStyle::Setext)
            .link_style(LinkStyle::Referenced)
            .bullet_marker('*')
            .code_fence('~')
            .base_url(Some("https://example.com".to_string()));

        assert_eq!(opts.heading_style, HeadingStyle::Setext);
        assert_eq!(opts.link_style, LinkStyle::Referenced);
        assert_eq!(opts.bullet_marker, '*');
        assert_eq!(opts.code_fence, '~');
        assert_eq!(opts.base_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_parse_styles() {
        assert_eq!("atx".parse::<HeadingStyle>(), Ok(HeadingStyle::Atx));
        assert_eq!("SETEXT".parse::<HeadingStyle>(), Ok(HeadingStyle::Setext));
        assert!("underline".parse::<HeadingStyle>().is_err());
        assert_eq!("inline".parse::<LinkStyle>(), Ok(LinkStyle::Inline));
        assert_eq!("reference".parse::<LinkStyle>(), Ok(LinkStyle::Referenced));
        assert_eq!("Referenced".parse::<LinkStyle>(), Ok(LinkStyle::Referenced));
        assert!("".parse::<LinkStyle>().is_err());
    }
}
