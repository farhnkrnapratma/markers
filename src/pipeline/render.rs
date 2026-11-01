use std::cell::Cell;

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
        if let Some(note) = self.annotations.get(&element.id()) {
            if note.excluded && !note.forced_include {
                return String::new();
            }
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
                    rendered.push_str(&escape_inline_text(&collapse_whitespace(text)));
                }
                Node::Element(_) => {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        rendered.push_str(&self.element(child_element));
                    }
                }
                _ => {}
            }
        }

        rendered
    }
}

fn plain_text(element: ElementRef) -> String {
    element
        .text()
        .map(|text| escape_inline_text(&collapse_whitespace(text)))
        .collect()
}
