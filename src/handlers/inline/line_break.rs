use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};

pub struct LineBreakHandler;

impl TagHandler for LineBreakHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["br"]
    }

    fn render(&self, _element: ElementRef, _cx: &RenderContext) -> String {
        "  \n".to_string()
    }
}
