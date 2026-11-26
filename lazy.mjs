import init, { convert } from "./wasm/markers.js";

let ready;

export function load(input) {
  ready ??= init(input === undefined ? undefined : { module_or_path: input }).then(
    () => undefined,
    (error) => {
      ready = undefined;
      throw error;
    },
  );
  return ready;
}

export { convert };

export async function convertAsync(html, options) {
  await load();
  return convert(html, options);
}
