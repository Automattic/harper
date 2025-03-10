import type { Lint, Span, Suggestion, Linter as WasmLinter } from 'harper-wasm';
import { Language } from 'harper-wasm';
import LazyPromise from 'p-lazy';
import Linter, { LinterInit } from './Linter';
import { LintConfig, LintOptions } from './main';
import { BinaryModule } from './binary';

/** A Linter that runs in the current JavaScript context (meaning it is allowed to block the event loop).  */
export default class LocalLinter implements Linter {
	binary: BinaryModule;
	private inner: Promise<WasmLinter>;

	constructor(init: LinterInit) {
		this.binary = init.binary;
		this.inner = LazyPromise.from(async () => {
			await this.binary.setup();
			return this.binary.createLinter();
		});
	}

	async setup(): Promise<void> {
		await this.lint('', { language: 'plaintext' });

		const exported = await this.exportIgnoredLints();
		await this.importIgnoredLints(exported);
	}

	async lint(text: string, options?: LintOptions): Promise<Lint[]> {
		const inner = await this.inner;
		const language = options?.language === 'plaintext' ? Language.Plain : Language.Markdown;
		const lints = inner.lint(text, language);

		return lints;
	}

	async applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string> {
		return await this.binary.applySuggestion(text, suggestion, span);
	}

	async isLikelyEnglish(text: string): Promise<boolean> {
		const inner = await this.inner;
		return inner.is_likely_english(text);
	}

	async isolateEnglish(text: string): Promise<string> {
		const inner = await this.inner;
		return inner.isolate_english(text);
	}

	async getLintConfig(): Promise<LintConfig> {
		const inner = await this.inner;
		return inner.get_lint_config_as_object();
	}

	async getDefaultLintConfigAsJSON(): Promise<string> {
		return this.binary.getDefaultLintConfigAsJSON();
	}

	async getDefaultLintConfig(): Promise<LintConfig> {
		return this.binary.getDefaultLintConfig();
	}

	async setLintConfig(config: LintConfig): Promise<void> {
		const inner = await this.inner;
		inner.set_lint_config_from_object(config);
	}

	async getLintConfigAsJSON(): Promise<string> {
		const inner = await this.inner;
		return inner.get_lint_config_as_json();
	}

	async setLintConfigWithJSON(config: string): Promise<void> {
		const inner = await this.inner;
		inner.set_lint_config_from_json(config);
	}

	async toTitleCase(text: string): Promise<string> {
		return this.binary.toTitleCase(text);
	}

	async getLintDescriptions(): Promise<Record<string, string>> {
		const inner = await this.inner;
		return inner.get_lint_descriptions_as_object();
	}

	async getLintDescriptionsAsJSON(): Promise<string> {
		const inner = await this.inner;
		return inner.get_lint_descriptions_as_json();
	}

	async ignoreLint(lint: Lint): Promise<void> {
		const inner = await this.inner;
		inner.ignore_lint(lint);
	}

	async exportIgnoredLints(): Promise<string> {
		const inner = await this.inner;
		return inner.export_ignored_lints();
	}

	async importIgnoredLints(json: string): Promise<void> {
		const inner = await this.inner;
		inner.import_ignored_lints(json);
	}

	async clearIgnoredLints(): Promise<void> {
		const inner = await this.inner;
		inner.clear_ignored_lints();
	}

	async importWords(words: string[]): Promise<void> {
		const inner = await this.inner;

		return inner.import_words(words);
	}

	async exportWords(): Promise<string[]> {
		const inner = await this.inner;

		return inner.export_words();
	}
}
