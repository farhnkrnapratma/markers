use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};

pub struct DefinitionListHandler;

impl TagHandler for DefinitionListHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["dl"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let mut result = String::from("\n\n");
        let mut last_was_dt = false;

        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                match el.value().name() {
                    "dt" => {
                        let content = cx.children(el);
                        let content = content.trim();
                        if !content.is_empty() {
                            if !last_was_dt && !result.trim().is_empty() {
                                result.push('\n');
                            }
                            result.push_str(content);
                            result.push('\n');
                            last_was_dt = true;
                        }
                    }
                    "dd" => {
                        let content = cx.children(el);
                        let content = content.trim();
                        if !content.is_empty() {
                            let lines: Vec<&str> = content.lines().collect();
                            for (i, line) in lines.iter().enumerate() {
                                if i == 0 {
                                    result.push_str(": ");
                                    result.push_str(line);
                                } else {
                                    result.push_str("\n  ");
                                    result.push_str(line);
                                }
                            }
                            result.push('\n');
                            last_was_dt = false;
                        }
                    }
                    _ => {}
                }
            }
        }

        result.push('\n');
        result
    }
}

pub struct DefinitionTermHandler;

impl TagHandler for DefinitionTermHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["dt"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let content = cx.children(element);
        content.trim().to_string()
    }
}

pub struct DefinitionDescriptionHandler;

impl TagHandler for DefinitionDescriptionHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["dd"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let content = cx.children(element);
        let content = content.trim();
        if content.is_empty() {
            String::new()
        } else {
            format!(": {}", content)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render;

    fn convert_test(html: &str) -> String {
        render(&DefinitionListHandler, html)
    }

    #[test]
    fn test_simple_deflist() {
        let result = convert_test(
            r#"<dl>
                <dt>Term</dt>
                <dd>Definition</dd>
            </dl>"#,
        );
        assert!(result.contains("Term"));
        assert!(result.contains(": Definition"));
    }

    #[test]
    fn test_multiple_definitions() {
        let result = convert_test(
            r#"<dl>
                <dt>Term 1</dt>
                <dd>Definition 1</dd>
                <dt>Term 2</dt>
                <dd>Definition 2</dd>
            </dl>"#,
        );
        assert!(result.contains("Term 1"));
        assert!(result.contains(": Definition 1"));
        assert!(result.contains("Term 2"));
        assert!(result.contains(": Definition 2"));
    }

    #[test]
    fn test_multiple_dd_per_dt() {
        let result = convert_test(
            r#"<dl>
                <dt>Term</dt>
                <dd>First definition</dd>
                <dd>Second definition</dd>
            </dl>"#,
        );
        assert!(result.contains("Term"));
        assert!(result.contains(": First definition"));
        assert!(result.contains(": Second definition"));
    }

    #[test]
    fn test_empty_deflist() {
        let result = convert_test("<dl></dl>");
        assert!(result.trim().is_empty());
    }
}
