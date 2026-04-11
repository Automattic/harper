// WASM benchmark harness for harper spell-check.
//
// Measures time and throughput for fuzzy_match
// across the same word lists used by the native Criterion benchmarks.
//
// Usage:
//   wasm-pack build --target web --no-opt --out-name harper_wasm --features bench
//   node harper-wasm/benches/wasm_bench.js

import { readFileSync } from "node:fs";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const pkgDir = join(__dirname, "..", "pkg");
const wordsDir = join(
  __dirname,
  "..",
  "..",
  "harper-core",
  "benches",
  "misspelled_words",
);

// Load the wasm-pack generated JS glue and raw wasm bytes.
const wasmModule = await import(join(pkgDir, "harper_wasm.js"));
const wasmBytes = readFileSync(join(pkgDir, "harper_wasm_bg.wasm"));
wasmModule.initSync({ module: wasmBytes });

const MAX_EDIT_DISTANCE = 3;
const MAX_RESULTS = 200;
const WARMUP_ITERS = 2;
const BENCH_ITERS = 10;

function bench(name, fn) {
  // Warmup.
  for (let i = 0; i < WARMUP_ITERS; i++) fn();

  const start = performance.now();
  let result;
  for (let i = 0; i < BENCH_ITERS; i++) {
    result = fn();
  }
  const elapsed = performance.now() - start;

  const avgMs = elapsed / BENCH_ITERS;
  console.log(`${name}:`);
  console.log(`  avg:     ${avgMs.toFixed(2)} ms/iter  (${BENCH_ITERS} iters)`);
  console.log(`  results: ${result}`);
  console.log();
}

// Same word lists as the native benchmarks.
const cases = [
  ["mixed", readFileSync(join(wordsDir, "mixed.md"), "utf-8")],
  ["lowercase", readFileSync(join(wordsDir, "lowercase.md"), "utf-8")],
  ["capitalized", readFileSync(join(wordsDir, "capitalized.md"), "utf-8")],
];

console.log("--- wasm fuzzy_match benchmark ---");
console.log(
  `max_edit_distance: ${MAX_EDIT_DISTANCE}, max_results: ${MAX_RESULTS}`,
);
console.log();

for (const [name, words] of cases) {
  const wordCount = words
    .split("\n")
    .filter((l) => l.length > 0).length;

  bench(`fuzzy_match/${name} (${wordCount} words)`, () =>
    wasmModule.bench_fuzzy_match(words, MAX_EDIT_DISTANCE, MAX_RESULTS),
  );
}
