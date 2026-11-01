use scraper::{ElementRef, Html};

use super::{RenderContext, TagHandler};
use crate::options::ConversionOptions;
use crate::pipeline::Annotations;

pub fn render(handler: &dyn TagHandler, html: &str) -> String {
    render_with(handler, html, &ConversionOptions::default())
}

pub fn render_with(handler: &dyn TagHandler, html: &str, options: &ConversionOptions) -> String {
    let dom = Html::parse_fragment(html);
    let element = ElementRef::wrap(dom.root_element().first_child().unwrap()).unwrap();
    let annotations = Annotations::default();
    let plain = |e: ElementRef| e.text().collect::<Vec<_>>().join("");
    handler.render(element, &RenderContext::new(options, &annotations, &plain))
}

pub fn with_annotated_element<R>(
    html: &str,
    selector: &str,
    options: &ConversionOptions,
    f: impl FnOnce(ElementRef, &Annotations) -> R,
) -> R {
    let dom = Html::parse_document(html);
    let annotations = crate::pipeline::annotate_tree(
        &dom,
        &crate::pipeline::ElementFilter::new(options),
        options,
    );
    let selector = scraper::Selector::parse(selector).unwrap();
    let element = dom.select(&selector).next().unwrap();
    f(element, &annotations)
}
