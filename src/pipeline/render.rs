use std::cell::Cell;

use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

use super::annotate::{Annotations, MAX_RENDER_DEPTH};
use crate::handlers::{HandlerRegistry, RenderContext};
use crate::options::ConversionOptions;
use crate::text::escape::escape_inline_text;
use crate::text::whitespace::collapse_whitespace;

pub(super) struct RenderPass<'a> {
    handlers: &'a HandlerRegistry,
    annotations: &'a Annotations,
    options: &'a ConversionOptions,
    depth: Cell<usize>,
}

impl<'a> RenderPass<'a> {
    pub(super) fn new(
        handlers: &'a HandlerRegistry,
        annotations: &'a Annotations,
        options: &'a ConversionOptions,
    ) -> Self {
        Self {
            handlers,
            annotations,
            options,
            depth: Cell::new(0),
        }
    }

    pub(super) fn element(&self, element: ElementRef) -> String {
        if let Some(note) = self.annotations.get(&element.id())
            && note.excluded
            && !note.forced_include
        {
            return String::new();
        }

        let depth = self.depth.get();
        if depth >= MAX_RENDER_DEPTH {
            return plain_text(element);
        }
        self.depth.set(depth + 1);

        let rendered = match self.handlers.for_tag(element.value().name()) {
            Some(handler) => {
                let render_children = |child: ElementRef| self.children(child);
                let cx = RenderContext::new(self.options, self.annotations, &render_children);
                handler.render(element, &cx)
            }
            None => self.children(element),
        };

        self.depth.set(depth);
        rendered
    }

    fn children(&self, element: ElementRef) -> String {
        let mut rendered = String::new();

        for child in element.children() {
            match child.value() {
                Node::Text(text) => {
                    if text.trim().is_empty() && touches_block(element, child) {
                        continue;
                    }
                    rendered.push_str(&escape_inline_text(&collapse_whitespace(text)));
                }
                Node::Element(_) => {
                    rendered.push_str(
                        &self.element(ElementRef::wrap(child).expect("an element node wraps")),
                    );
                }
                _ => {}
            }
        }

        rendered
    }
}

const BLOCK_TAGS: [&str; 37] = [
    "address",
    "article",
    "aside",
    "blockquote",
    "body",
    "dd",
    "details",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tr",
    "ul",
    "html",
];

fn is_block(node: NodeRef<Node>) -> bool {
    node.value()
        .as_element()
        .is_some_and(|element| BLOCK_TAGS.contains(&element.name()))
}

fn touches_block(parent: ElementRef, text: NodeRef<Node>) -> bool {
    let boundary = |sibling: Option<NodeRef<Node>>| match sibling {
        Some(node) => is_block(node),
        None => is_block(*parent),
    };
    let previous = text.prev_siblings().find(|node| !node.value().is_comment());
    let next = text.next_siblings().find(|node| !node.value().is_comment());
    boundary(previous) || boundary(next)
}

fn plain_text(element: ElementRef) -> String {
    element
        .text()
        .map(|text| escape_inline_text(&collapse_whitespace(text)))
        .collect()
}
