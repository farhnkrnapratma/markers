use super::RawOptions;
use crate::ConversionOptions as CoreOptions;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[derive(Default)]
#[napi(object)]
pub struct ConversionOptions {
    #[napi(ts_type = "'atx' | 'setext'")]
    pub heading_style: Option<String>,
    #[napi(ts_type = "'inline' | 'referenced'")]
    pub link_style: Option<String>,
    #[napi(ts_type = "'`' | '~'")]
    pub code_fence: Option<String>,
    #[napi(ts_type = "'-' | '*' | '+'")]
    pub bullet_marker: Option<String>,
    pub base_url: Option<String>,
    pub exclude_selectors: Option<Vec<String>>,
    pub include_selectors: Option<Vec<String>>,
}

fn convert_catching_panics(html: &str, opts: &CoreOptions) -> std::result::Result<String, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::convert_with_options(html, opts)
    }))
    .map_err(|panic_info| {
        let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = panic_info.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "unknown panic".to_string()
        };
        format!("conversion panicked: {msg}")
    })
}

#[napi]
pub fn convert(html: String, options: Option<ConversionOptions>) -> Result<String> {
    let opts = parse_options(options).map_err(Error::from_reason)?;
    convert_catching_panics(&html, &opts).map_err(Error::from_reason)
}

impl From<ConversionOptions> for RawOptions {
    fn from(opts: ConversionOptions) -> Self {
        Self {
            heading_style: opts.heading_style,
            link_style: opts.link_style,
            code_fence: opts.code_fence,
            bullet_marker: opts.bullet_marker,
            base_url: opts.base_url,
            exclude_selectors: opts.exclude_selectors,
            include_selectors: opts.include_selectors,
        }
    }
}

fn parse_options(opts: Option<ConversionOptions>) -> std::result::Result<CoreOptions, String> {
    RawOptions::from(opts.unwrap_or_default()).into_options()
}

pub struct ConvertTask {
    html: String,
    options: std::result::Result<CoreOptions, String>,
}

#[napi]
impl Task for ConvertTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> Result<Self::Output> {
        let options = self.options.as_ref().map_err(Error::from_reason)?;
        convert_catching_panics(&self.html, options).map_err(Error::from_reason)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[cfg_attr(test, allow(dead_code))]
#[napi]
pub fn convert_async(html: String, options: Option<ConversionOptions>) -> AsyncTask<ConvertTask> {
    AsyncTask::new(ConvertTask {
        html,
        options: parse_options(options),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_basic() {
        let html = "<h1>Hello</h1><p>World</p>";
        let result = convert(html.to_string(), None).unwrap();
        assert!(result.contains("# Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_convert_with_options() {
        let html = "<h1>Title</h1>";
        let options = ConversionOptions {
            heading_style: Some("setext".to_string()),
            link_style: None,
            code_fence: None,
            bullet_marker: None,
            base_url: None,
            exclude_selectors: None,
            include_selectors: None,
        };
        let result = convert(html.to_string(), Some(options)).unwrap();
        assert!(result.contains("====="));
    }

    #[test]
    fn test_convert_with_exclude() {
        let html = "<div><nav>Skip</nav><p>Keep</p></div>";
        let options = ConversionOptions {
            heading_style: None,
            link_style: None,
            code_fence: None,
            bullet_marker: None,
            base_url: None,
            exclude_selectors: Some(vec!["nav".to_string()]),
            include_selectors: None,
        };
        let result = convert(html.to_string(), Some(options)).unwrap();
        assert!(!result.contains("Skip"));
        assert!(result.contains("Keep"));
    }

    #[test]
    fn test_convert_empty_input() {
        let result = convert(String::new(), None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_invalid_heading_style_is_rejected() {
        let options = ConversionOptions {
            heading_style: Some("setex".to_string()),
            ..Default::default()
        };
        let err = convert("<h1>x</h1>".to_string(), Some(options)).unwrap_err();
        assert!(err.reason.contains("headingStyle"));
    }

    #[test]
    fn test_invalid_link_style_is_rejected() {
        let options = ConversionOptions {
            link_style: Some("footnote".to_string()),
            ..Default::default()
        };
        assert!(convert("<p>x</p>".to_string(), Some(options)).is_err());
    }

    #[test]
    fn test_invalid_code_fence_and_bullet_are_rejected() {
        for (fence, bullet) in [
            (Some(""), None),
            (Some("x"), None),
            (None, Some("")),
            (None, Some("o")),
        ] {
            let options = ConversionOptions {
                code_fence: fence.map(String::from),
                bullet_marker: bullet.map(String::from),
                ..Default::default()
            };
            assert!(convert("<p>x</p>".to_string(), Some(options)).is_err());
        }
    }

    #[test]
    fn test_valid_style_values_are_case_insensitive() {
        let options = ConversionOptions {
            heading_style: Some("SETEXT".to_string()),
            link_style: Some("Reference".to_string()),
            code_fence: Some("~".to_string()),
            bullet_marker: Some("*".to_string()),
            ..Default::default()
        };
        let result = convert(
            "<h1>Title</h1><ul><li>a</li></ul>".to_string(),
            Some(options),
        )
        .unwrap();
        assert!(result.contains("====="));
        assert!(result.contains("* a"));
    }

    #[test]
    fn test_async_task_reports_invalid_options() {
        let options = ConversionOptions {
            heading_style: Some("nope".to_string()),
            ..Default::default()
        };
        let mut task = ConvertTask {
            html: "<h1>x</h1>".to_string(),
            options: parse_options(Some(options)),
        };
        assert!(task.compute().is_err());
    }

    #[test]
    fn test_async_task_converts() {
        let mut task = ConvertTask {
            html: "<h1>x</h1>".to_string(),
            options: parse_options(None),
        };
        assert_eq!(task.compute().unwrap(), "# x");
    }
}
