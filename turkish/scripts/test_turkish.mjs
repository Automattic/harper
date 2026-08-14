import { Linter, Dialect, Language, setup } from '../../harper-wasm/pkg-node/harper_wasm.js';

setup();

const linter = Linter.new(Dialect.American);
const text = "Kısa özet olarak, İLK ÖNCE bunu geri iade etmemiz lazım.";

const lints = linter.lint(text, Language.Plain, false, undefined, true, false);

console.log('Girdi:', text);
console.log('Bulunan', lints.length, 'hata:');
for (const lint of lints) {
  const span = lint.span();
  const suggestions = lint.suggestions().map((s) => s.get_replacement_text());
  console.log(`  "${lint.get_problem_text()}" [${span.start},${span.end}] (${lint.lint_kind()}): ${lint.message()} -> ${suggestions.join(', ')}`);
}
