use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};

pub struct ListHandler;

impl TagHandler for ListHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["ul", "ol"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let content = cx.children(element);
        let content = content.trim_end();

        if content.is_empty() {
            return String::new();
        }

        format!("\n\n{}\n\n", content)
    }
}

pub struct ListItemHandler;

impl TagHandler for ListItemHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["li"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let content = cx.children(element);
        let content = content.trim();

        if content.is_empty() {
            return String::new();
        }

        let (prefix, indent): (String, usize) =
            if let Some(meta) = cx.annotations.get(&element.id()) {
                (
                    meta.list_marker.clone().unwrap_or_else(|| "- ".to_string()),
                    meta.list_indent,
                )
            } else {
                (format!("{} ", cx.options.bullet_marker), 0)
            };

        let task_checkbox = detect_task_checkbox(&element);
        let (final_prefix, final_content) = if let Some(checked) = task_checkbox {
            let checkbox_marker = if checked { "[x] " } else { "[ ] " };
            let cleaned = content
                .trim_start_matches("[ ]")
                .trim_start_matches("[x]")
                .trim_start();
            (
                format!("{}{}", prefix, checkbox_marker),
                cleaned.to_string(),
            )
        } else {
            (prefix.clone(), content.to_string())
        };

        let indented = indent_continuation(&final_content, final_prefix.len() + indent);

        format!("{}{}{}\n", " ".repeat(indent), final_prefix, indented)
    }
}

fn detect_task_checkbox(li: &ElementRef) -> Option<bool> {
    use once_cell::sync::Lazy;
    use scraper::Selector;

    static INPUT_SEL: Lazy<Selector> =
        Lazy::new(|| Selector::parse("input[type='checkbox']").unwrap());

    li.select(&INPUT_SEL)
        .next()
        .map(|input| input.value().attr("checked").is_some())
}

fn indent_continuation(text: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    let mut lines = text.lines();

    let first = lines.next().unwrap_or("");
    let rest: String = lines
        .map(|line| {
            if line.is_empty() {
                "\n".to_string()
            } else {
                format!("\n{}{}", indent, line)
            }
        })
        .collect();

    format!("{}{}", first, rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::with_annotated_element;
    use crate::options::ConversionOptions;
    use crate::pipeline::Annotations;
    use scraper::Html;

    fn render_item_element(
        li: ElementRef,
        annotations: &Annotations,
        options: &ConversionOptions,
    ) -> String {
        let plain = |e: ElementRef| e.text().collect::<Vec<_>>().join("");
        ListItemHandler.render(li, &RenderContext::new(options, annotations, &plain))
    }

    fn render_item(html: &str, options: &ConversionOptions) -> String {
        with_annotated_element(html, "li", options, |li, annotations| {
            render_item_element(li, annotations, options)
        })
    }

    fn render_list(html: &str, selector: &str, options: &ConversionOptions) -> String {
        with_annotated_element(html, selector, options, |list, annotations| {
            let items = |e: ElementRef| {
                e.children()
                    .filter_map(ElementRef::wrap)
                    .filter(|child| child.value().name() == "li")
                    .map(|li| render_item_element(li, annotations, options))
                    .collect::<String>()
            };
            ListHandler.render(list, &RenderContext::new(options, annotations, &items))
        })
    }

    #[test]
    fn test_unordered_list() {
        let result = render_list(
            "<ul><li>First</li><li>Second</li></ul>",
            "ul",
            &ConversionOptions::default(),
        );
        assert!(result.contains("- First"));
        assert!(result.contains("- Second"));
    }

    #[test]
    fn test_ordered_list() {
        let result = render_list(
            "<ol><li>One</li><li>Two</li><li>Three</li></ol>",
            "ol",
            &ConversionOptions::default(),
        );
        assert!(result.contains("1. One"));
        assert!(result.contains("2. Two"));
        assert!(result.contains("3. Three"));
    }

    #[test]
    fn test_custom_bullet() {
        let options = ConversionOptions::new().bullet_marker('*');
        assert!(render_item("<ul><li>Item</li></ul>", &options).contains("* Item"));
    }

    #[test]
    fn test_indent_continuation() {
        let text = "Line 1\nLine 2\nLine 3";
        let result = indent_continuation(text, 4);
        assert_eq!(result, "Line 1\n    Line 2\n    Line 3");
    }

    #[test]
    fn test_ordered_list_with_start() {
        let result = render_list(
            r#"<ol start="5"><li>Fifth</li><li>Sixth</li><li>Seventh</li></ol>"#,
            "ol",
            &ConversionOptions::default(),
        );
        assert!(result.contains("5. Fifth"));
        assert!(result.contains("6. Sixth"));
        assert!(result.contains("7. Seventh"));
    }

    #[test]
    fn test_nested_list() {
        let html = r#"<ul><li>Level 1<ul><li>Level 2</li></ul></li></ul>"#;
        let dom = Html::parse_document(html);
        let options = ConversionOptions::default();
        let annotations = crate::pipeline::annotate_tree(
            &dom,
            &crate::pipeline::ElementFilter::new(&options),
            &options,
        );

        let items: Vec<_> = annotations
            .values()
            .filter(|note| note.list_marker.is_some())
            .collect();

        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|note| note.list_indent > 0));
    }

    #[test]
    fn test_empty_list_items_skipped() {
        let result = render_list(
            "<ul><li>Item 1</li><li></li><li>Item 3</li></ul>",
            "ul",
            &ConversionOptions::default(),
        );
        assert!(result.contains("- Item 1"));
        assert!(result.contains("- Item 3"));
        assert_eq!(result.matches("- ").count(), 2);
    }

    #[test]
    fn test_plus_bullet_marker() {
        let options = ConversionOptions::new().bullet_marker('+');
        assert!(render_item("<ul><li>Item</li></ul>", &options).contains("+ Item"));
    }

    #[test]
    fn test_task_list_unchecked() {
        let result = render_item(
            r#"<ul><li><input type="checkbox"> Buy milk</li></ul>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("- [ ] Buy milk"), "got: {}", result);
    }

    #[test]
    fn test_task_list_checked() {
        let result = render_item(
            r#"<ul><li><input type="checkbox" checked> Done task</li></ul>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("- [x] Done task"), "got: {}", result);
    }

    #[test]
    fn test_task_list_checked_equals_checked() {
        let result = render_item(
            r#"<ul><li><input type="checkbox" checked="checked"> Also done</li></ul>"#,
            &ConversionOptions::default(),
        );
        assert!(result.contains("- [x] Also done"), "got: {}", result);
    }

    #[test]
    fn test_regular_list_item_no_checkbox() {
        let result = render_item(
            "<ul><li>Regular item</li></ul>",
            &ConversionOptions::default(),
        );
        assert!(result.contains("- Regular item"), "got: {}", result);
        assert!(!result.contains("[ ]"));
        assert!(!result.contains("[x]"));
    }

    fn first_li(dom: &Html) -> ElementRef<'_> {
        dom.select(&scraper::Selector::parse("li").unwrap())
            .next()
            .unwrap()
    }

    #[test]
    fn test_detect_task_checkbox_none() {
        let dom = Html::parse_fragment("<li>No checkbox</li>");
        assert_eq!(detect_task_checkbox(&first_li(&dom)), None);
    }

    #[test]
    fn test_detect_task_checkbox_unchecked() {
        let dom = Html::parse_fragment(r#"<li><input type="checkbox"> Task</li>"#);
        assert_eq!(detect_task_checkbox(&first_li(&dom)), Some(false));
    }

    #[test]
    fn test_detect_task_checkbox_checked() {
        let dom = Html::parse_fragment(r#"<li><input type="checkbox" checked> Task</li>"#);
        assert_eq!(detect_task_checkbox(&first_li(&dom)), Some(true));
    }
}
