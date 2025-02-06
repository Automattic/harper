import type { Lint, Span, Suggestion, Linter as WasmLinter } from 'harper-wasm';
import { Language } from 'harper-wasm';
import Linter, { LinterInit } from './Linter';
import { LintConfig, LintOptions } from './main';
import { binary, loadBinary } from './binary';

/** A Linter that runs in the current JavaScript context (meaning it is allowed to block the event loop).  */
export default class LocalLinter implements Linter {
	private binary: string;
	private inner: WasmLinter | undefined;

	constructor(init: LinterInit) {
		this.binary = init.binary;
	}

	/** Initialize the WebAssembly and construct the inner Linter. */
	private async initialize(): Promise<void> {
		if (!this.inner) {
			const exports = await loadBinary(this.binary);
			exports.setup();
			this.inner = exports.Linter.new();
		}
	}

	async setup(): Promise<void> {
		await this.initialize();
		this.inner!.lint('', Language.Plain);
	}

	async lint(text: string, options?: LintOptions): Promise<Lint[]> {
		await this.initialize();
		const lints = this.inner!.lint(
			text,
			options?.language === 'plaintext' ? Language.Plain : Language.Markdown
		);

		return lints;
	}

	async applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string> {
		const wasm = await loadBinary(binary);
		return wasm.apply_suggestion(text, span, suggestion);
	}

	async isLikelyEnglish(text: string): Promise<boolean> {
		await this.initialize();
		return this.inner!.is_likely_english(text);
	}

	async isolateEnglish(text: string): Promise<string> {
		await this.initialize();
		return this.inner!.isolate_english(text);
	}

	async getLintConfig(): Promise<LintConfig> {
		await this.initialize();

		return this.inner!.get_lint_config_as_object();
	}

	async getDefaultLintConfigAsJSON(): Promise<string> {
		const wasm = await loadBinary(binary);

		return wasm.get_default_lint_config_as_json();
	}

	async getDefaultLintConfig(): Promise<LintConfig> {
		const wasm = await loadBinary(binary);

		return wasm.get_default_lint_config();
	}

	async setLintConfig(config: LintConfig): Promise<void> {
		await this.initialize();

		this.inner!.set_lint_config_from_object(config);
	}

	async getLintConfigAsJSON(): Promise<string> {
		await this.initialize();

		return this.inner!.get_lint_config_as_json();
	}

	async setLintConfigWithJSON(config: string): Promise<void> {
		await this.initialize();

		this.inner!.set_lint_config_from_json(config);
	}

	async toTitleCase(text: string): Promise<string> {
		const wasm = await loadBinary(binary);
		return wasm.to_title_case(text);
	}

	async getLintDescriptions(): Promise<Record<string, string>> {
		await this.initialize();
		return this.inner!.get_lint_descriptions_as_object();
	}

	async getLintDescriptionsAsJSON(): Promise<string> {
		await this.initialize();
		return this.inner!.get_lint_descriptions_as_json();
	}

	async ignoreLint(lint: Lint): Promise<void> {
		await this.initialize();

		this.inner!.ignore_lint(lint);
	}

	async exportIgnoredLints(): Promise<string> {
		await this.initialize();

		return this.inner!.export_ignored_lints();
	}

	async importIgnoredLints(json: string): Promise<void> {
		await this.initialize();

		return this.inner!.import_ignored_lints(json);
	}

	async clearIgnoredLints(): Promise<void> {
		await this.initialize();

		return this.inner!.clear_ignored_lints();
	}
}
