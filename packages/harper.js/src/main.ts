export type { Lint, Span, Suggestion } from 'harper-wasm';
export { Dialect, SuggestionKind } from 'harper-wasm';
export {
	BinaryModule,
	binary,
	binaryInlined,
	type DeserializedRequest,
	isSerializedRequest,
	type RequestArg,
	type SerializableTypes,
	type SerializedRequest,
} from './binary';
export type { default as Linter, LinterInit } from './Linter';
export { default as LocalLinter } from './LocalLinter';
export type { default as Summary } from './Summary';
export { default as WorkerLinter } from './WorkerLinter';
/** A linting rule configuration dependent on upstream Harper's available rules.
 * This is a record, since you shouldn't hard-code the existence of any particular rules and should generalize based on this struct. */
export type LintConfig = Record<string, boolean | null>;

/** The option used to configure the parser for an individual linting operation. */
export interface LintOptions {
	/** The markup language that is being passed. Defaults to `markdown`. */
	language?: 'plaintext' | 'markdown';
}
