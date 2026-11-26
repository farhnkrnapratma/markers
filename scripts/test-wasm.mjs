import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const native = createRequire(import.meta.url)("../index.js");
const wasm = await import("../lazy.mjs");

await wasm.load(readFileSync(join(root, "wasm/markers_bg.wasm")));

const optionSets = [
  undefined,
  null,
  {},
  { headingStyle: "setext", linkStyle: "referenced", codeFence: "~", bulletMarker: "*" },
  { baseUrl: "https://example.com/docs/", excludeSelectors: ["footer", ".ads"] },
  { excludeSelectors: ["body"], includeSelectors: ["article", "pre"] },
];
const fixtures = readdirSync(join(root, "tests/fixtures")).map((name) =>
  readFileSync(join(root, "tests/fixtures", name), "utf8"),
);
const inputs = [
  ...fixtures,
  "",
  "<p>café \u{1F600} <a href='javascript:alert(1)'>x</a></p>",
  "<ul><li>a<ul><li>b</li></ul></li></ul><table><tr><th>h</th></tr><tr><td>c</td></tr></table>",
];

let checks = 0;
for (const html of inputs)
  for (const options of optionSets) {
    assert.equal(wasm.convert(html, options), native.convert(html, options));
    assert.equal(await wasm.convertAsync(html, options), await native.convertAsync(html, options));
    checks += 2;
  }

for (const options of [{ excludeSelectors: ["::nope("] }, { headingStyle: "bogus" }]) {
  assert.throws(() => native.convert("<p>x</p>", options));
  assert.throws(() => wasm.convert("<p>x</p>", options));
  checks += 2;
}

console.log(`wasm matches the native addon on ${checks} checks`);
