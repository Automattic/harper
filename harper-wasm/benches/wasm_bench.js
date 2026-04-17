// WASM benchmark harness for harper spell-check.
//
// Measures time and throughput for fuzzy_match
// across the same word lists used by the native Criterion benchmarks.
//
// Run via `just bench-wasm`.

import { readFileSync } from "node:fs";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
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

// Node lacks the fetch-based init that --target web expects, so load the
// wasm bytes manually and pass them to initSync. We read the bench-specific
// build (harper_wasm_bench.*) so the shipping pkg/harper_wasm.* stays clean.
const wasmModule = await import(pathToFileURL(join(pkgDir, "harper_wasm_bench.js")).href);
const wasmBytes = readFileSync(join(pkgDir, "harper_wasm_bench_bg.wasm"));
wasmModule.initSync({ module: wasmBytes });

const MAX_EDIT_DISTANCE = 3;
const MAX_RESULTS = 200;
// Warmup lets V8 tier wasm up to TurboFan before measurement. min is the most
// useful single number for microbenches because it's least affected by GC or
// scheduler jitter; avg is printed alongside so the spread is visible.
const WARMUP_ITERS = 5;
const BENCH_ITERS = 50;

function bench(name, fn) {
  for (let i = 0; i < WARMUP_ITERS; i++) fn();

  const times = [];
  let result;
  for (let i = 0; i < BENCH_ITERS; i++) {
    const start = performance.now();
    result = fn();
    times.push(performance.now() - start);
  }

  const min = Math.min(...times);
  const avg = times.reduce((a, b) => a + b, 0) / times.length;

  console.log(`${name}:`);
  console.log(`  min:     ${min.toFixed(2)} ms`);
  console.log(`  avg:     ${avg.toFixed(2)} ms  (${BENCH_ITERS} iters)`);
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

  // Pre-parse outside the timed region so we measure fuzzy_match, not parse +
  // JS->WASM string encoding.
  const prepared = new wasmModule.PreparedWords(words);

  bench(`fuzzy_match/${name} (${wordCount} words)`, () =>
    prepared.bench_fuzzy_match(MAX_EDIT_DISTANCE, MAX_RESULTS),
  );
  // wasm-bindgen needs explicit free() — wasm-pack --target web doesn't
  // enable FinalizationRegistry auto-cleanup.
  prepared.free();
}
