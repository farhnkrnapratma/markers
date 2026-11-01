use scraper::ElementRef;

use crate::handlers::{RenderContext, TagHandler};
use crate::text::escape::escape_inline_text;
use crate::text::whitespace::collapse_whitespace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Alignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

struct CellData {
    content: String,
    alignment: Alignment,
}

pub struct TableHandler;

impl TagHandler for TableHandler {
    fn handled_tags(&self) -> &'static [&'static str] {
        &["table"]
    }

    fn render(&self, element: ElementRef, cx: &RenderContext) -> String {
        let mut rows: Vec<Vec<CellData>> = Vec::new();
        let mut caption: Option<String> = None;

        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                match el.value().name() {
                    "caption" => {
                        let text = cx.children(el);
                        let text = collapse_whitespace(text.trim());
                        if !text.is_empty() {
                            caption = Some(text);
                        }
                    }
                    "thead" => {
                        extract_rows(&el, cx, &mut rows);
                    }
                    "tbody" | "tfoot" => {
                        extract_rows(&el, cx, &mut rows);
                    }
                    "tr" => {
                        if let Some(row) = extract_row(&el, cx) {
                            rows.push(row);
                        }
                    }
                    _ => {}
                }
            }
        }

        if rows.is_empty() {
            return String::new();
        }

        let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut col_widths: Vec<usize> = vec![3; col_count];
        let mut col_alignments: Vec<Alignment> = vec![Alignment::None; col_count];

        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.content.chars().count());
                    if col_alignments[i] == Alignment::None && cell.alignment != Alignment::None {
                        col_alignments[i] = cell.alignment;
                    }
                }
            }
        }

        let mut result = String::from("\n\n");

        for (row_idx, row) in rows.iter().enumerate() {
            result.push('|');
            for (col_idx, cell) in row.iter().enumerate() {
                let width = col_widths.get(col_idx).copied().unwrap_or(3);
                let alignment = col_alignments
                    .get(col_idx)
                    .copied()
                    .unwrap_or(Alignment::None);

                let char_count = cell.content.chars().count();
                let padding = width.saturating_sub(char_count);
                let formatted = match alignment {
                    Alignment::Right => {
                        format!(" {}{} |", " ".repeat(padding), cell.content)
                    }
                    Alignment::Center => {
                        let left_pad = padding / 2;
                        let right_pad = padding - left_pad;
                        format!(
                            " {}{}{} |",
                            " ".repeat(left_pad),
                            cell.content,
                            " ".repeat(right_pad)
                        )
                    }
                    _ => {
                        format!(" {}{} |", cell.content, " ".repeat(padding))
                    }
                };
                result.push_str(&formatted);
            }
            for col_idx in row.len()..col_count {
                let width = col_widths.get(col_idx).copied().unwrap_or(3);
                result.push_str(&format!(" {:width$} |", "", width = width));
            }
            result.push('\n');

            if row_idx == 0 {
                result.push('|');
                for (col_idx, width) in col_widths.iter().enumerate() {
                    let alignment = col_alignments
                        .get(col_idx)
                        .copied()
                        .unwrap_or(Alignment::None);
                    let w = *width;
                    let separator = match alignment {
                        Alignment::Left => {
                            format!(" :{} |", "-".repeat(w.saturating_sub(1).max(2)))
                        }
                        Alignment::Center => {
                            format!(" :{}: |", "-".repeat(w.saturating_sub(2).max(1)))
                        }
                        Alignment::Right => {
                            format!(" {}: |", "-".repeat(w.saturating_sub(1).max(2)))
                        }
                        Alignment::None => format!(" {} |", "-".repeat(w.max(3))),
                    };
                    result.push_str(&separator);
                }
                result.push('\n');
            }
        }

        if let Some(cap) = caption {
            result.push_str(&format!("\n*{}*", cap));
        }

        result.push('\n');
        result
    }
}

fn extract_rows(container: &ElementRef, cx: &RenderContext, rows: &mut Vec<Vec<CellData>>) {
    for child in container.children() {
        if let Some(el) = ElementRef::wrap(child) {
            if el.value().name() == "tr" {
                if let Some(row) = extract_row(&el, cx) {
                    rows.push(row);
                }
            }
        }
    }
}

fn extract_row(tr: &ElementRef, cx: &RenderContext) -> Option<Vec<CellData>> {
    let mut cells = Vec::new();

    for child in tr.children() {
        if let Some(el) = ElementRef::wrap(child) {
            let tag = el.value().name();
            if tag == "th" || tag == "td" {
                let content = if contains_table(&el) {
                    el.text()
                        .map(escape_inline_text)
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    cx.children(el)
                };
                let content = collapse_whitespace(content.trim());
                let content = content.replace('|', "\\|");

                let alignment = extract_alignment(&el);

                cells.push(CellData { content, alignment });
            }
        }
    }

    if cells.is_empty() {
        None
    } else {
        Some(cells)
    }
}

fn contains_table(cell: &ElementRef) -> bool {
    cell.descendants()
        .any(|n| n.value().as_element().is_some_and(|e| e.name() == "table"))
}

fn extract_alignment(element: &ElementRef) -> Alignment {
    if let Some(align) = element.value().attr("align") {
        return match align.to_lowercase().as_str() {
            "left" => Alignment::Left,
            "center" => Alignment::Center,
            "right" => Alignment::Right,
            _ => Alignment::None,
        };
    }

    if let Some(style) = element.value().attr("style") {
        let style_lower = style.to_lowercase();
        if style_lower.contains("text-align:") || style_lower.contains("text-align :") {
            if style_lower.contains("left") {
                return Alignment::Left;
            } else if style_lower.contains("center") {
                return Alignment::Center;
            } else if style_lower.contains("right") {
                return Alignment::Right;
            }
        }
    }

    Alignment::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::testing::render;

    fn convert_test(html: &str) -> String {
        render(&TableHandler, html)
    }

    #[test]
    fn test_simple_table() {
        let result = convert_test(
            r#"<table>
                <tr><th>Header 1</th><th>Header 2</th></tr>
                <tr><td>Cell 1</td><td>Cell 2</td></tr>
            </table>"#,
        );
        assert!(result.contains("| Header 1"));
        assert!(result.contains("| Cell 1"));
        assert!(result.contains("---"));
    }

    #[test]
    fn test_table_with_thead_tbody() {
        let result = convert_test(
            r#"<table>
                <thead><tr><th>Name</th><th>Age</th></tr></thead>
                <tbody><tr><td>Alice</td><td>30</td></tr></tbody>
            </table>"#,
        );
        assert!(result.contains("| Name"));
        assert!(result.contains("| Alice"));
    }

    #[test]
    fn test_table_with_pipes() {
        let result = convert_test(
            r#"<table>
                <tr><th>A</th></tr>
                <tr><td>a | b</td></tr>
            </table>"#,
        );
        assert!(result.contains("a \\| b"));
    }

    #[test]
    fn test_empty_table() {
        let result = convert_test("<table></table>");
        assert!(result.is_empty());
    }

    #[test]
    fn test_table_with_alignment() {
        let result = convert_test(
            r#"<table>
                <tr>
                    <th align="left">Left</th>
                    <th align="center">Center</th>
                    <th align="right">Right</th>
                </tr>
                <tr>
                    <td align="left">L</td>
                    <td align="center">C</td>
                    <td align="right">R</td>
                </tr>
            </table>"#,
        );
        assert!(result.contains(":---"));
        assert!(result.contains(":----:"));
        assert!(result.contains("----:"));
    }

    #[test]
    fn test_table_with_caption() {
        let result = convert_test(
            r#"<table>
                <caption>Monthly Sales</caption>
                <tr><th>Month</th><th>Sales</th></tr>
                <tr><td>Jan</td><td>$100</td></tr>
            </table>"#,
        );
        assert!(result.contains("| Month"));
        assert!(result.contains("*Monthly Sales*"));
    }

    #[test]
    fn test_table_with_style_alignment() {
        let result = convert_test(
            r#"<table>
                <tr>
                    <th style="text-align: right">Right</th>
                </tr>
                <tr>
                    <td style="text-align: right">R</td>
                </tr>
            </table>"#,
        );
        assert!(result.contains("---:"));
    }

    #[test]
    fn test_table_with_missing_cells() {
        let result = convert_test(
            r#"<table>
                <tr><th>A</th><th>B</th><th>C</th></tr>
                <tr><td>1</td><td>2</td></tr>
                <tr><td>X</td></tr>
            </table>"#,
        );
        let lines: Vec<&str> = result.lines().filter(|l| l.contains('|')).collect();
        assert!(lines.len() >= 3);

        let header_pipes = lines.first().map(|l| l.matches('|').count()).unwrap_or(0);
        for line in &lines {
            assert_eq!(line.matches('|').count(), header_pipes);
        }
    }

    #[test]
    fn test_table_with_tfoot() {
        let result = convert_test(
            r#"<table>
                <thead><tr><th>Item</th><th>Price</th></tr></thead>
                <tbody><tr><td>Apple</td><td>$1</td></tr></tbody>
                <tfoot><tr><td>Total</td><td>$1</td></tr></tfoot>
            </table>"#,
        );
        assert!(result.contains("| Item"));
        assert!(result.contains("| Apple"));
        assert!(result.contains("| Total"));
    }

    #[test]
    fn test_table_header_only() {
        let result = convert_test(
            r#"<table>
                <tr><th>Col A</th><th>Col B</th></tr>
            </table>"#,
        );
        assert!(result.contains("| Col A"));
        assert!(result.contains("---"));
    }

    #[test]
    fn test_table_single_cell() {
        let result = convert_test("<table><tr><td>Only</td></tr></table>");
        assert!(result.contains("| Only"));
    }

    #[test]
    fn test_table_empty_cells() {
        let result = convert_test(
            r#"<table>
                <tr><th>A</th><th>B</th></tr>
                <tr><td></td><td></td></tr>
            </table>"#,
        );
        assert!(result.contains("---"));
    }

    #[test]
    fn test_table_more_data_cols_than_header() {
        let result = convert_test(
            r#"<table>
                <tr><th>A</th></tr>
                <tr><td>1</td><td>2</td><td>3</td></tr>
            </table>"#,
        );
        assert!(result.contains("| A"));
        assert!(result.contains("| 1"));
    }

    #[test]
    fn test_table_wide_100_columns() {
        let mut html = String::from("<table><tr>");
        for i in 0..100 {
            html.push_str(&format!("<th>H{}</th>", i));
        }
        html.push_str("</tr><tr>");
        for i in 0..100 {
            html.push_str(&format!("<td>D{}</td>", i));
        }
        html.push_str("</tr></table>");

        let result = convert_test(&html);
        assert!(result.contains("| H0"));
        assert!(result.contains("| H99"));
        assert!(result.contains("| D0"));
        assert!(result.contains("| D99"));
    }

    #[test]
    fn test_table_unicode_content() {
        let result = convert_test(
            r#"<table>
                <tr><th>名前</th><th>説明</th></tr>
                <tr><td>太郎</td><td>テスト</td></tr>
            </table>"#,
        );
        assert!(result.contains("名前"));
        assert!(result.contains("太郎"));
    }

    #[test]
    fn test_table_with_only_empty_rows() {
        let result = convert_test(
            r#"<table>
                <tr></tr>
                <tr></tr>
            </table>"#,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn test_table_nested_html_in_cells() {
        let result = convert_test(
            r#"<table>
                <tr><th>Link</th></tr>
                <tr><td><a href="https://example.com">Click</a></td></tr>
            </table>"#,
        );
        assert!(result.contains("Click"));
    }

    #[test]
    fn test_table_with_braces_in_content() {
        let result = convert_test(
            r#"<table>
                <tr><th>Syntax</th><th>Example</th></tr>
                <tr><td>Object</td><td>{key: value}</td></tr>
                <tr><td>Template</td><td>{{variable}}</td></tr>
                <tr><td>Set</td><td>{1, 2, 3}</td></tr>
            </table>"#,
        );
        assert!(result.contains("{key: value}"));
        assert!(result.contains("{{variable}}"));
        assert!(result.contains("{1, 2, 3}"));
    }

    #[test]
    fn test_table_with_format_like_content() {
        let result = convert_test(
            r#"<table>
                <tr><th>Pattern</th></tr>
                <tr><td>{:>10}</td></tr>
                <tr><td>{0:.2}</td></tr>
                <tr><td>{name}</td></tr>
            </table>"#,
        );
        assert!(result.contains("{:>10}"));
        assert!(result.contains("{0:.2}"));
        assert!(result.contains("{name}"));
    }

    #[test]
    fn test_table_with_mixed_th_td_in_body() {
        let result = convert_test(
            r#"<table>
                <tr><th>Header</th></tr>
                <tr><th>Also TH</th></tr>
                <tr><td>Regular TD</td></tr>
            </table>"#,
        );
        assert!(result.contains("| Header"));
        assert!(result.contains("| Also TH"));
        assert!(result.contains("| Regular TD"));
    }
}
