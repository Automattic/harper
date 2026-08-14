import { Linter, Language, setup } from '../../harper-wasm/pkg-node/harper_wasm.js';

setup();

const linter = Linter.new_turkish();

const text = "Bana birşey söyleme, çünkü onunda haberi var; kısa özet olarak, eve geldimi haber ver.";
const lints = linter.lint(text, Language.Plain, false, undefined, true, false);

console.log('Girdi:', text);
console.log('Bulunan', lints.length, 'hata:\n');
for (const lint of lints) {
  const suggestions = lint.suggestions().map((s) => s.get_replacement_text());
  console.log(`  "${lint.get_problem_text()}" (${lint.lint_kind()}) -> ${suggestions.join(', ')}`);
}
