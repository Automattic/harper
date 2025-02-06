import type { Lint, Span, Suggestion } from 'harper-wasm';
import { LintConfig, LintOptions } from './main';

/** An interface for an object that can perform linting actions. */
export default interface Linter {
	/** Complete any setup that is necessary before linting. This may include downloading and compiling the WebAssembly binary.
	 * This setup will complete when needed regardless of whether you call this function.
	 * This function exists to allow you to do this work when it is of least impact to the user experiences (i.e. while you're loading something else). */
	setup(): Promise<void>;

	/** Lint the provided text. */
	lint(text: string, options?: LintOptions): Promise<Lint[]>;

	/** Apply a suggestion to the given text, returning the transformed result. */
	applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string>;

	/** Determine if the provided text is likely to be intended to be English.
	 * The algorithm can be described as "proof of concept" and as such does not work terribly well.*/
	isLikelyEnglish(text: string): Promise<boolean>;

	/** Determine which parts of a given string are intended to be English, returning those bits.
	 * The algorithm can be described as "proof of concept" and as such does not work terribly well.*/
	isolateEnglish(text: string): Promise<string>;

	/** Get the linter's current configuration. */
	getLintConfig(): Promise<LintConfig>;

	/** Get the default (unset) linter configuration as JSON.
	 * This method does not effect the caller's lint configuration, nor does it return the current one. */
	getDefaultLintConfigAsJSON(): Promise<string>;

	/** Get the default (unset) linter configuration.
	 * This method does not effect the caller's lint configuration, nor does it return the current one. */
	getDefaultLintConfig(): Promise<LintConfig>;

	/** Set the linter's current configuration. */
	setLintConfig(config: LintConfig): Promise<void>;

	/** Get the linter's current configuration as JSON. */
	getLintConfigAsJSON(): Promise<string>;

	/** Set the linter's current configuration from JSON. */
	setLintConfigWithJSON(config: string): Promise<void>;

	/** Get the linting rule descriptions as a JSON map. */
	getLintDescriptionsAsJSON(): Promise<string>;

	/** Get the linting rule descriptions as an object */
	getLintDescriptions(): Promise<Record<string, string>>;

	/** Convert a string to Chicago-style title case. */
	toTitleCase(text: string): Promise<string>;

	/** Ignore future instances of a lint from a previous linting run in future invocations. */
	ignoreLint(lint: Lint): Promise<void>;

	/** Export the ignored lints to a JSON list of privacy-respecting hashes. */
	exportIgnoredLints(): Promise<string>;

	/** Import ignored lints from a JSON list to the linter.
	 * This function appends to the existing lints, if any. */
	importIgnoredLints(json: string): Promise<void>;

	/** Clear records of all previously ignored lints. */
	clearIgnoredLints(): Promise<void>;
}

export interface LinterInit {
	/** The module or path to the WebAssembly binary. */
	binary: string;
}
