fn main() {
    let html = r#"
        <h1>Release notes</h1>
        <p>Version <strong>0.1</strong> adds <em>GFM</em> support. See the
        <a href="https://example.com/docs">docs</a>.</p>
        <ul>
            <li>Tables</li>
            <li>Task lists</li>
        </ul>
        <table>
            <tr><th>Crate</th><th>Purpose</th></tr>
            <tr><td>markers</td><td>Rust library</td></tr>
            <tr><td>markers-napi</td><td>Node.js bindings</td></tr>
        </table>
        <pre><code class="language-rust">fn main() {}</code></pre>
    "#;

    println!("{}", markers::convert(html));
}
