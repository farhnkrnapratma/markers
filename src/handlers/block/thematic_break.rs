use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};

pub struct ThematicBreakHandler;

impl TagHandler for ThematicBreakHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["hr"]
    }

    fn render(&self, _element: ElementRef, _cx: &RenderContext) -> String {
        "\n\n---\n\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render;

    #[test]
    fn test_hr() {
        assert_eq!(render(&ThematicBreakHandler, "<hr>"), "\n\n---\n\n");
    }
}
