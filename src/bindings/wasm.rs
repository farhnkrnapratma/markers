use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::RawOptions;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionOptions {
    pub heading_style: Option<String>,
    pub link_style: Option<String>,
    pub code_fence: Option<String>,
    pub bullet_marker: Option<String>,
    pub base_url: Option<String>,
    pub exclude_selectors: Option<Vec<String>>,
    pub include_selectors: Option<Vec<String>>,
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

#[wasm_bindgen]
pub fn convert(html: &str) -> String {
    crate::convert(html)
}

#[wasm_bindgen(js_name = convertWithOptions)]
pub fn convert_with_options(html: &str, options: JsValue) -> Result<String, JsError> {
    let opts: Option<ConversionOptions> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options)?)
    };

    let internal_opts = RawOptions::from(opts.unwrap_or_default())
        .into_options()
        .map_err(|e| JsError::new(&e))?;
    Ok(crate::convert_with_options(html, &internal_opts))
}
