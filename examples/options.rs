use markers::{ConversionOptions, HeadingStyle, LinkStyle, convert_with_options};

fn main() -> Result<(), String> {
    let html = r#"
        <nav>Home | Blog</nav>
        <h1>Guide</h1>
        <ul><li>First</li><li>Second</li></ul>
        <p>Read the <a href="/docs/intro">intro</a>.</p>
        <p class="ad">Buy now!</p>
    "#;

    let options = ConversionOptions::new()
        .heading_style(HeadingStyle::Setext)
        .bullet_marker('*')
        .link_style(LinkStyle::Referenced)
        .base_url(Some("https://example.com".to_string()))
        .exclude_selectors(vec!["nav".to_string(), ".ad".to_string()]);

    println!("{}", convert_with_options(html, &options)?);
    Ok(())
}
