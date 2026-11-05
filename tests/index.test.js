const test = require('ava').default
const { convert, convertAsync } = require('../index.js')

test('converts basic markup', (t) => {
  t.is(convert('<h1>Title</h1><p>Hello <strong>world</strong></p>'), '# Title\n\nHello **world**')
})

test('returns an empty string for empty input', (t) => {
  t.is(convert(''), '')
})

test('collapses indentation between blocks to one blank line', (t) => {
  t.is(convert('<h1>A</h1>\n  <p>B</p>\n  <p>C</p>'), '# A\n\nB\n\nC')
})

test('applies heading and bullet options', (t) => {
  const out = convert('<h1>Title</h1><ul><li>one</li></ul>', {
    headingStyle: 'setext',
    bulletMarker: '*',
  })
  t.is(out, 'Title\n=====\n\n* one')
})

test('resolves relative links against baseUrl', (t) => {
  t.is(convert('<a href="/x">x</a>', { baseUrl: 'https://example.com' }), '[x](https://example.com/x)')
})

test('drops elements matching excludeSelectors', (t) => {
  t.is(convert('<nav>menu</nav><p>body</p>', { excludeSelectors: ['nav'] }), 'body')
})

test('flattens a table nested in a cell instead of blowing up', (t) => {
  const depth = 40
  const html = '<table><tr><td>'.repeat(depth) + 'deep' + '</td></tr></table>'.repeat(depth)
  const out = convert(html)
  t.true(out.includes('deep'))
  t.true(out.length < 1000)
})

test('convertAsync matches convert', async (t) => {
  const html = '<h2>Async</h2>'
  t.is(await convertAsync(html), convert(html))
})
