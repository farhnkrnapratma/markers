use crate::handlers::HandlerRegistry;
use crate::pipeline::Converter;
use crate::text::url::resolve_against_base;
use crate::{ConversionOptions, HeadingStyle, convert, convert_with_options};

#[test]
fn definition_lists_skip_empty_and_foreign_children() {
    assert_eq!(
        convert("<dl><dt></dt><dd></dd><div>x</div><dt>T</dt><dd>a<br>b</dd></dl>"),
        "T\n: a\\\n  b"
    );
    assert_eq!(convert("<dt>Term</dt><dd>Def</dd><dd></dd>"), "Term: Def");
}

#[test]
fn details_ignore_comments() {
    assert_eq!(
        convert("<details><!--c--><summary>s</summary>x</details>"),
        "> **s**\n>\n> x"
    );
}

#[test]
fn figures_find_nested_images() {
    assert_eq!(
        convert(
            "<figure><div><img src=\"a.png\" alt=\"b\"></div><figcaption>c</figcaption></figure>"
        ),
        "![b](a.png)\n*c*"
    );
    assert_eq!(
        convert("<figure><img src=\"a.png\" alt=\"b\"><span>x</span></figure>"),
        "![b](a.png)"
    );
}

#[test]
fn setext_headings_fall_back_to_atx_below_level_three() {
    let options = ConversionOptions::default().heading_style(HeadingStyle::Setext);
    assert_eq!(
        convert_with_options("<h3>x</h3>", &options).unwrap(),
        "### x"
    );
}

#[test]
fn lists_handle_empty_blank_and_unmarked_items() {
    assert_eq!(convert("<ul></ul>"), "");
    assert_eq!(convert("<ul><li><p>a</p><p>b</p></li></ul>"), "- a\n\n  b");
    let options = ConversionOptions::default().include_selectors(vec!["li".into()]);
    assert_eq!(convert_with_options("<li>x</li>", &options).unwrap(), "- x");
}

#[test]
fn preformatted_blocks_ignore_comments() {
    assert_eq!(convert("<pre><!-- c -->code</pre>"), "```\ncode\n```");
}

#[test]
fn tables_read_alignment_and_skip_foreign_cells() {
    assert_eq!(
        convert(
            "<table><caption> </caption><tr><td align=\"justify\">a</td><td style=\"text-align:left\">b</td><td style=\"text-align:center\">c</td><td style=\"text-align:justify\">d</td><td style=\"color:red\">e</td><script></script></tr></table>"
        ),
        "| a   | b   |  c  | d   | e   |\n| --- | :-- | :-: | --- | --- |"
    );
    assert_eq!(
        convert("<table><colgroup></colgroup><tr><td>a<td>b</table>"),
        "| a   | b   |\n| --- | --- |"
    );
}

#[test]
fn links_without_text_or_label_stay_empty() {
    assert_eq!(convert("<a href=\"http://x.com\"></a>"), "[](http://x.com)");
}

#[test]
fn defaults_match_the_constructors() {
    assert!(HandlerRegistry::default().for_tag("p").is_some());
    let converter = Converter::default();
    assert_eq!(
        converter
            .convert("<p>x</p>", &ConversionOptions::default())
            .unwrap(),
        "x"
    );
}

#[test]
fn resolves_relative_urls_against_bases_without_an_origin() {
    assert_eq!(resolve_against_base("example", "/x"), "example/x");
    assert_eq!(resolve_against_base("example", "x"), "example/x");
}

#[test]
fn nested_exclusions_keep_the_outermost_root() {
    let options = ConversionOptions::default().exclude_selectors(vec!["div".into(), "p".into()]);
    assert_eq!(
        convert_with_options("<div><p>a</p></div><span>b</span>", &options).unwrap(),
        "b"
    );
}
