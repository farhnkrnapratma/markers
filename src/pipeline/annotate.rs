use ego_tree::NodeId;
use rustc_hash::FxHashMap;
use scraper::{ElementRef, Html, Selector};

use crate::options::ConversionOptions;

#[derive(Debug, Default, Clone)]
pub struct NodeAnnotation {
    pub list_marker: Option<String>,

    pub list_indent: usize,

    pub excluded: bool,

    pub forced_include: bool,
}

pub type Annotations = FxHashMap<NodeId, NodeAnnotation>;

pub struct ElementFilter {
    pub exclude: Vec<Selector>,
    pub include: Vec<Selector>,
}

impl ElementFilter {
    pub fn new(options: &ConversionOptions) -> Self {
        Self {
            exclude: options
                .exclude_selectors
                .iter()
                .filter_map(|s| compile_selector(s))
                .collect(),
            include: options
                .include_selectors
                .iter()
                .filter_map(|s| compile_selector(s))
                .collect(),
        }
    }

    pub fn excludes(&self, element: &ElementRef) -> bool {
        self.exclude.iter().any(|sel| sel.matches(element))
    }

    pub fn includes(&self, element: &ElementRef) -> bool {
        self.include.iter().any(|sel| sel.matches(element))
    }
}

fn compile_selector(selector: &str) -> Option<Selector> {
    Selector::parse(selector).ok()
}

struct OpenList {
    ordered: bool,
    index: usize,
    indent: usize,
    prefix_len: usize,
}

pub const MAX_RENDER_DEPTH: usize = 256;

pub fn annotate_tree(
    dom: &Html,
    filter: &ElementFilter,
    options: &ConversionOptions,
) -> Annotations {
    let mut annotations = FxHashMap::default();
    let mut open_lists: Vec<OpenList> = Vec::with_capacity(8);
    let mut exclusion_root_depth: Option<usize> = None;
    let mut depth: usize = 0;

    let root = dom.root_element();

    fn visit(
        node: ego_tree::NodeRef<scraper::Node>,
        annotations: &mut Annotations,
        open_lists: &mut Vec<OpenList>,
        exclusion_root_depth: &mut Option<usize>,
        depth: &mut usize,
        filter: &ElementFilter,
        options: &ConversionOptions,
    ) {
        *depth += 1;

        if let Some(element) = ElementRef::wrap(node) {
            let tag = element.value().name();

            if tag == "ul" || tag == "ol" {
                let current_indent = open_lists
                    .last()
                    .map(|ctx| ctx.indent + ctx.prefix_len)
                    .unwrap_or(0);

                let start_index = if tag == "ol" {
                    element
                        .value()
                        .attr("start")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(1)
                        .saturating_sub(1)
                } else {
                    0
                };

                open_lists.push(OpenList {
                    ordered: tag == "ol",
                    index: start_index,
                    indent: current_indent,
                    prefix_len: 2,
                });
            }

            if tag == "li" {
                if let Some(ctx) = open_lists.last_mut() {
                    ctx.index += 1;

                    let marker = if ctx.ordered {
                        format!("{}. ", ctx.index)
                    } else {
                        format!("{} ", options.bullet_marker)
                    };

                    ctx.prefix_len = marker.len();

                    let note = annotations.entry(node.id()).or_default();
                    note.list_marker = Some(marker);
                    note.list_indent = ctx.indent;
                }
            }

            let forced_include = filter.includes(&element);

            let excludes = filter.excludes(&element);

            let inside_excluded = exclusion_root_depth.is_some();
            let excluded = if forced_include {
                false
            } else if excludes {
                if exclusion_root_depth.is_none() {
                    *exclusion_root_depth = Some(*depth);
                }
                true
            } else {
                inside_excluded
            };

            if excluded || forced_include {
                let note = annotations.entry(node.id()).or_default();
                note.excluded = excluded;
                note.forced_include = forced_include;
            }
        }

        if *depth < MAX_RENDER_DEPTH {
            for child in node.children() {
                visit(
                    child,
                    annotations,
                    open_lists,
                    exclusion_root_depth,
                    depth,
                    filter,
                    options,
                );
            }
        }

        if let Some(element) = ElementRef::wrap(node) {
            let tag = element.value().name();
            if tag == "ul" || tag == "ol" {
                open_lists.pop();
            }
        }

        if *exclusion_root_depth == Some(*depth) {
            *exclusion_root_depth = None;
        }
        *depth -= 1;
    }

    for child in root.children() {
        visit(
            child,
            &mut annotations,
            &mut open_lists,
            &mut exclusion_root_depth,
            &mut depth,
            filter,
            options,
        );
    }

    annotations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_markers() {
        let html = r#"<ul><li>First</li><li>Second</li></ul>"#;
        let dom = Html::parse_document(html);
        let options = ConversionOptions::default();
        let filter = ElementFilter::new(&options);
        let annotations = annotate_tree(&dom, &filter, &options);

        let item_notes: Vec<_> = annotations
            .values()
            .filter(|m| m.list_marker.is_some())
            .collect();
        assert_eq!(item_notes.len(), 2);
    }

    #[test]
    fn test_nested_list_indentation() {
        let html = r#"<ul><li>First<ul><li>Nested</li></ul></li></ul>"#;
        let dom = Html::parse_document(html);
        let options = ConversionOptions::default();
        let filter = ElementFilter::new(&options);
        let annotations = annotate_tree(&dom, &filter, &options);

        let item_notes: Vec<_> = annotations
            .values()
            .filter(|m| m.list_marker.is_some())
            .collect();

        let has_nested = item_notes.iter().any(|m| m.list_indent > 0);
        assert!(has_nested);
    }

    #[test]
    fn test_ordered_list() {
        let html = r#"<ol><li>First</li><li>Second</li><li>Third</li></ol>"#;
        let dom = Html::parse_document(html);
        let options = ConversionOptions::default();
        let filter = ElementFilter::new(&options);
        let annotations = annotate_tree(&dom, &filter, &options);

        let markers: Vec<_> = annotations
            .values()
            .filter_map(|m| m.list_marker.as_ref())
            .collect();

        assert!(markers.contains(&&"1. ".to_string()));
        assert!(markers.contains(&&"2. ".to_string()));
        assert!(markers.contains(&&"3. ".to_string()));
    }

    #[test]
    fn test_ordered_list_with_start() {
        let html = r#"<ol start="5"><li>Fifth</li><li>Sixth</li><li>Seventh</li></ol>"#;
        let dom = Html::parse_document(html);
        let options = ConversionOptions::default();
        let filter = ElementFilter::new(&options);
        let annotations = annotate_tree(&dom, &filter, &options);

        let markers: Vec<_> = annotations
            .values()
            .filter_map(|m| m.list_marker.as_ref())
            .collect();

        assert!(markers.contains(&&"5. ".to_string()));
        assert!(markers.contains(&&"6. ".to_string()));
        assert!(markers.contains(&&"7. ".to_string()));
    }

    #[test]
    fn test_exclude_selector() {
        let html = r#"<div><p>Keep</p><nav>Skip</nav></div>"#;
        let dom = Html::parse_document(html);
        let options = ConversionOptions {
            exclude_selectors: vec!["nav".to_string()],
            ..Default::default()
        };
        let filter = ElementFilter::new(&options);
        let annotations = annotate_tree(&dom, &filter, &options);

        let excluded_notes: Vec<_> = annotations.values().filter(|m| m.excluded).collect();
        assert!(!excluded_notes.is_empty());
    }

    #[test]
    fn test_include_overrides_exclude() {
        let html = r#"<nav><div class="keep">Important</div></nav>"#;
        let dom = Html::parse_document(html);
        let options = ConversionOptions {
            exclude_selectors: vec!["nav".to_string()],
            include_selectors: vec![".keep".to_string()],
            ..Default::default()
        };
        let filter = ElementFilter::new(&options);
        let annotations = annotate_tree(&dom, &filter, &options);

        let force_kept: Vec<_> = annotations.values().filter(|m| m.forced_include).collect();
        assert!(!force_kept.is_empty());
    }
}
