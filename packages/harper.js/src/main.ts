export type { Lint, Span, Suggestion } from 'harper-wasm';
export { Dialect, SuggestionKind } from 'harper-wasm';
export {
	BinaryModule,
	binary,
	binaryInlined,
} from './binary';
export type {
	default as Linter,
	LinterInit,
	WeirpackTestFailure,
	WeirpackTestFailures,
} from './Linter';
export { default as LocalLinter } from './LocalLinter';
export type { default as Summary } from './Summary';
export { default as WorkerLinter } from './WorkerLinter';
export type { WeirpackArchive } from './weirpack';
export { packWeirpackFiles, unpackWeirpackBytes } from './weirpack';
/** A linting rule configuration dependent on upstream Harper's available rules.
 * This is a record, since you shouldn't hard-code the existence of any particular rules and should generalize based on this struct. */
export type LintConfig = Record<string, boolean | null>;

/**  Options available to configure Harper's parser for an individual linting operation. */
export interface LintOptions {
	/** The markup language that is being passed. Defaults to `markdown`. */
	language?: 'plaintext' | 'markdown' | 'typst';
  regex_mask?: string;

	/** Force the entirety of the document to be composed of headings. An undefined value is assumed to be false.*/
	forceAllHeadings?: boolean;
}
