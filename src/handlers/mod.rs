mod block;
mod inline;
#[cfg(test)]
pub(crate) mod testing;

use rustc_hash::FxHashMap;
use scraper::ElementRef;

use crate::options::ConversionOptions;
use crate::pipeline::Annotations;

use block::{
    BlockquoteHandler, DefinitionDescriptionHandler, DefinitionListHandler, DefinitionTermHandler,
    DetailsHandler, FigureHandler, HeadingHandler, ListHandler, ListItemHandler,
    PreformattedHandler, TableHandler, ThematicBreakHandler,
};
use inline::{
    AbbreviationHandler, ImageHandler, InlineCodeHandler, LineBreakHandler, LinkHandler,
    WrapHandler,
};

pub trait TagHandler: Send + Sync {
    fn handled_tags(&self) -> &'static [&'static str];

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String;
}

pub struct RenderContext<'a> {
    pub options: &'a ConversionOptions,
    pub annotations: &'a Annotations,
    render_children: &'a dyn Fn(ElementRef) -> String,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        options: &'a ConversionOptions,
        annotations: &'a Annotations,
        render_children: &'a dyn Fn(ElementRef) -> String,
    ) -> Self {
        Self {
            options,
            annotations,
            render_children,
        }
    }

    pub fn children(&self, element: ElementRef) -> String {
        (self.render_children)(element)
    }
}

pub struct HandlerRegistry {
    handlers: Vec<Box<dyn TagHandler>>,
    by_tag: FxHashMap<&'static str, usize>,
}

impl HandlerRegistry {
    pub fn standard() -> Self {
        let handlers: Vec<Box<dyn TagHandler>> = vec![
            Box::new(HeadingHandler),
            Box::new(WrapHandler::PARAGRAPH),
            Box::new(PreformattedHandler),
            Box::new(BlockquoteHandler),
            Box::new(ListHandler),
            Box::new(ListItemHandler),
            Box::new(DefinitionListHandler),
            Box::new(DefinitionTermHandler),
            Box::new(DefinitionDescriptionHandler),
            Box::new(TableHandler),
            Box::new(ThematicBreakHandler),
            Box::new(DetailsHandler),
            Box::new(FigureHandler),
            Box::new(LinkHandler),
            Box::new(ImageHandler),
            Box::new(WrapHandler::STRONG),
            Box::new(WrapHandler::EMPHASIS),
            Box::new(WrapHandler::STRIKETHROUGH),
            Box::new(InlineCodeHandler),
            Box::new(WrapHandler::SUPERSCRIPT),
            Box::new(WrapHandler::SUBSCRIPT),
            Box::new(LineBreakHandler),
            Box::new(WrapHandler::KEYBOARD),
            Box::new(WrapHandler::MARK),
            Box::new(AbbreviationHandler),
            Box::new(WrapHandler::SAMPLE_OUTPUT),
            Box::new(WrapHandler::VARIABLE),
        ];

        let mut by_tag = FxHashMap::default();
        for (index, handler) in handlers.iter().enumerate() {
            for &tag in handler.handled_tags() {
                by_tag.entry(tag).or_insert(index);
            }
        }

        Self { handlers, by_tag }
    }

    pub fn for_tag(&self, tag: &str) -> Option<&dyn TagHandler> {
        self.by_tag
            .get(tag)
            .map(|&index| self.handlers[index].as_ref())
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_handled_tag_resolves_to_its_handler() {
        let registry = HandlerRegistry::standard();
        for handler in &registry.handlers {
            for tag in handler.handled_tags() {
                let found = registry.for_tag(tag).expect("tag is registered");
                assert!(std::ptr::addr_eq(found, handler.as_ref()), "{tag}");
            }
        }
    }

    #[test]
    fn test_unknown_tag_has_no_handler() {
        assert!(HandlerRegistry::standard().for_tag("blink").is_none());
    }
}
