const { convert, convertAsync } = require('../index.js')

const html = `
  <nav>Home | Blog</nav>
  <h1>Release notes</h1>
  <p>Read the <a href="/docs/intro">intro</a>.</p>
  <ul><li>Tables</li><li>Task lists</li></ul>
  <p class="ad">Buy now!</p>
`

console.log(convert(html))
console.log('---')

console.log(
  convert(html, {
    headingStyle: 'setext',
    bulletMarker: '*',
    linkStyle: 'referenced',
    baseUrl: 'https://example.com',
    excludeSelectors: ['nav', '.ad'],
  }),
)
console.log('---')

convertAsync('<h2>Async</h2>').then(console.log)
