/** A version of the Harper WebAssembly binary stored inline as a data URL.
 * Can be tree-shaken if unused. */
export declare const binary: BinaryModule;

/** A version of the Harper WebAssembly binary stored inline as a data URL.
 * Can be tree-shaken if unused. */
export declare const binaryInlined: BinaryModule;

/** A wrapper around the underlying WebAssembly module that contains Harper's core code. Used to construct a `Linter`, as well as access some miscellaneous other functions. */
export declare class BinaryModule {
    url: string | URL;
    private inner;
    /** Load a binary from a specified URL. This is the only recommended way to construct this type. */
    static create(url: string | URL): BinaryModule;
    getDefaultLintConfigAsJSON(): Promise<string>;
    getDefaultLintConfig(): Promise<LintConfig>;
    toTitleCase(text: string): Promise<string>;
    setup(): Promise<void>;
}

/**
 * Specifies an English Dialect, often used for linting.
 */
export declare enum Dialect {
    American = 0,
    British = 1,
    Australian = 2,
    Canadian = 3,
}

declare enum Language {
    Plain = 0,
    Markdown = 1,
}

/**
 * An error found in provided text.
 *
 * May include zero or more suggestions that may fix the problematic text.
 */
export declare class Lint {
    private constructor();
    free(): void;
    /**
     * Get an array of any suggestions that may resolve the issue.
     */
    suggestions(): Suggestion[];
    /**
     * Get a description of the error as HTML.
     */
    message_html(): string;
    /**
     * Get the content of the source material pointed to by [`Self::span`]
     */
    get_problem_text(): string;
    /**
     * Get a string representing the general category of the lint.
     */
    lint_kind_pretty(): string;
    /**
     * Equivalent to calling `.length` on the result of `suggestions()`.
     */
    suggestion_count(): number;
    /**
     * Get the location of the problematic text.
     */
    span(): Span;
    /**
     * Get a description of the error.
     */
    message(): string;
    to_json(): string;
    static from_json(json: string): Lint;
    /**
     * Get a string representing the general category of the lint.
     */
    lint_kind(): string;
}

/** A linting rule configuration dependent on upstream Harper's available rules.
 * This is a record, since you shouldn't hard-code the existence of any particular rules and should generalize based on this struct. */
export declare type LintConfig = Record<string, boolean | null>;

/** An interface for an object that can perform linting actions. */
export declare interface Linter {
    /** Complete any setup that is necessary before linting. This may include downloading and compiling the WebAssembly binary.
     * This setup will complete when needed regardless of whether you call this function.
     * This function exists to allow you to do this work when it is of least impact to the user experiences (i.e. while you're loading something else). */
    setup(): Promise<void>;
    /** Lint the provided text. */
    lint(text: string, options?: LintOptions): Promise<Lint[]>;
    /** Lint the provided text, maintaining the relationship with the source rule. */
    organizedLints(text: string, options?: LintOptions): Promise<Record<string, Lint[]>>;
    /** Apply a suggestion from a lint to text, returning the changed text. */
    applySuggestion(text: string, lint: Lint, suggestion: Suggestion): Promise<string>;
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
    /** Get the linting rule descriptions as a JSON map, formatted in Markdown. */
    getLintDescriptionsAsJSON(): Promise<string>;
    /** Get the linting rule descriptions as an object, formatted in Markdown. */
    getLintDescriptions(): Promise<Record<string, string>>;
    /** Get the linting rule descriptions as a JSON map, formatted in HTML.
     * Wraps the function on the BinaryModule by the same name. */
    getLintDescriptionsHTMLAsJSON(): Promise<string>;
    /** Get the linting rule descriptions as an object, formatted in HTML.
     * Wraps the function on the BinaryModule by the same name. */
    getLintDescriptionsHTML(): Promise<Record<string, string>>;
    /** Convert a string to Chicago-style title case.
     Wraps the function on the BinaryModule by the same name. */
    toTitleCase(text: string): Promise<string>;
    /** Ignore future instances of a lint from a previous linting run in future invocations. */
    ignoreLint(source: string, lint: Lint): Promise<void>;
    /** Ignore future instances of a lint from a previous linting run in future invocations using its hash. */
    ignoreLintHash(hash: bigint): Promise<void>;
    /** Export the ignored lints to a JSON list of privacy-respecting hashes. */
    exportIgnoredLints(): Promise<string>;
    /** Import ignored lints from a JSON list to the linter.
     * This function appends to the existing lints, if any. */
    importIgnoredLints(json: string): Promise<void>;
    /** Produce a context-sensitive hash that represents a lint.  */
    contextHash(source: string, lint: Lint): Promise<bigint>;
    /** Clear records of all previously ignored lints. */
    clearIgnoredLints(): Promise<void>;
    /** Clear the words which have been added to the dictionary. This will not clear words from the curated dictionary. */
    clearWords(): Promise<void>;
    /** Import words into the dictionary. This is a significant operation, so try to batch words. */
    importWords(words: string[]): Promise<void>;
    /** Export all added words from the dictionary. Note that this will NOT export anything from the curated dictionary,
     * only words from previous calls to `this.importWords`. */
    exportWords(): Promise<string[]>;
    /** Get the dialect of English this linter was constructed for. */
    getDialect(): Promise<Dialect>;
    /** Get the dialect of English this linter was constructed for. */
    setDialect(dialect: Dialect): Promise<void>;
    /** Summarize the linter's usage statistics.
     * You may optionally pass in a start and/or end time.
     *
     * If so, the summary with only include data from _after_ the start time but _before_ the end time. */
    summarizeStats(start?: bigint, end?: bigint): Promise<Summary>;
    /** Generate a statistics log file you can save to permanent storage. */
    generateStatsFile(): Promise<string>;
    /** Import a statistics log file. */
    importStatsFile(statsFile: string): Promise<void>;
}

declare class Linter_2 {
    private constructor();
    free(): void;
    /**
     * Clear the user dictionary.
     */
    clear_words(): void;
    /**
     * Get the dialect this struct was constructed for.
     */
    get_dialect(): Dialect;
    /**
     * Add a specific context hash to the ignored lints list.
     */
    ignore_hash(hash: bigint): void;
    ignore_lint(source_text: string, lint: Lint): void;
    /**
     * Compute the context hash of a given lint.
     */
    context_hash(source_text: string, lint: Lint): bigint;
    /**
     * Export words from the dictionary.
     * Note: this will only return words previously added by [`Self::import_words`].
     */
    export_words(): string[];
    /**
     * Import words into the dictionary.
     */
    import_words(additional_words: string[]): void;
    /**
     * Helper method to remove non-English text from a plain English document.
     */
    isolate_english(text: string): string;
    organized_lints(text: string, language: Language, all_headings: boolean): OrganizedGroup[];
    summarize_stats(start_time?: bigint | null, end_time?: bigint | null): any;
    /**
     * Apply a suggestion from a given lint.
     * This action will be logged to the Linter's statistics.
     */
    apply_suggestion(source_text: string, lint: Lint, suggestion: Suggestion): string;
    import_stats_file(file: string): void;
    /**
     * Helper method to quickly check if a plain string is likely intended to be English
     */
    is_likely_english(text: string): boolean;
    clear_ignored_lints(): void;
    generate_stats_file(): string;
    /**
     * Export the linter's ignored lints as a privacy-respecting JSON list of hashes.
     */
    export_ignored_lints(): string;
    /**
     * Import into the linter's ignored lints from a privacy-respecting JSON list of hashes.
     */
    import_ignored_lints(json: string): void;
    get_lint_config_as_json(): string;
    get_lint_config_as_object(): any;
    set_lint_config_from_json(json: string): void;
    set_lint_config_from_object(object: any): void;
    /**
     * Get a JSON map containing the descriptions of all the linting rules, formatted as Markdown.
     */
    get_lint_descriptions_as_json(): string;
    /**
     * Get a Record containing the descriptions of all the linting rules, formatted as Markdown.
     */
    get_lint_descriptions_as_object(): any;
    /**
     * Get a JSON map containing the descriptions of all the linting rules, formatted as HTML.
     */
    get_lint_descriptions_html_as_json(): string;
    /**
     * Get a Record containing the descriptions of all the linting rules, formatted as HTML.
     */
    get_lint_descriptions_html_as_object(): any;
    /**
     * Construct a new `Linter`.
     * Note that this can mean constructing the curated dictionary, which is the most expensive operation
     * in Harper.
     */
    static new(dialect: Dialect): Linter_2;
    /**
     * Perform the configured linting on the provided text.
     */
    lint(text: string, language: Language, all_headings: boolean): Lint[];
}

/** The properties and information needed to construct a Linter. */
export declare interface LinterInit {
    /** The module or path to the WebAssembly binary. */
    binary: BinaryModule;
    /** The dialect of English Harper should use. If omitted, Harper will default to American English. */
    dialect?: Dialect;
}

/**  Options available to configure Harper's parser for an individual linting operation. */
export declare interface LintOptions {
    /** The markup language that is being passed. Defaults to `markdown`. */
    language?: 'plaintext' | 'markdown';
    /** Force the entirety of the document to be composed of headings. An undefined value is assumed to be false.*/
    forceAllHeadings?: boolean;
}

/** A Linter that runs in the current JavaScript context (meaning it is allowed to block the event loop).
 * See the interface definition for more details. */
export declare class LocalLinter implements Linter {
    binary: SuperBinaryModule;
    private inner;
    constructor(init: LinterInit);
    private createInner;
    setup(): Promise<void>;
    lint(text: string, options?: LintOptions): Promise<Lint[]>;
    organizedLints(text: string, options?: LintOptions): Promise<Record<string, Lint[]>>;
    applySuggestion(text: string, lint: Lint, suggestion: Suggestion): Promise<string>;
    isLikelyEnglish(text: string): Promise<boolean>;
    isolateEnglish(text: string): Promise<string>;
    getLintConfig(): Promise<LintConfig>;
    getDefaultLintConfigAsJSON(): Promise<string>;
    getDefaultLintConfig(): Promise<LintConfig>;
    setLintConfig(config: LintConfig): Promise<void>;
    getLintConfigAsJSON(): Promise<string>;
    setLintConfigWithJSON(config: string): Promise<void>;
    toTitleCase(text: string): Promise<string>;
    getLintDescriptions(): Promise<Record<string, string>>;
    getLintDescriptionsAsJSON(): Promise<string>;
    getLintDescriptionsHTML(): Promise<Record<string, string>>;
    getLintDescriptionsHTMLAsJSON(): Promise<string>;
    ignoreLint(source: string, lint: Lint): Promise<void>;
    ignoreLintHash(hash: bigint): Promise<void>;
    exportIgnoredLints(): Promise<string>;
    importIgnoredLints(json: string): Promise<void>;
    contextHash(source: string, lint: Lint): Promise<bigint>;
    clearIgnoredLints(): Promise<void>;
    clearWords(): Promise<void>;
    importWords(words: string[]): Promise<void>;
    exportWords(): Promise<string[]>;
    getDialect(): Promise<Dialect>;
    setDialect(dialect: Dialect): Promise<void>;
    summarizeStats(start?: bigint, end?: bigint): Promise<any>;
    generateStatsFile(): Promise<string>;
    importStatsFile(statsFile: string): Promise<void>;
}

/**
 * Used exclusively for [`Linter::organized_lints`]
 */
declare class OrganizedGroup {
    private constructor();
    free(): void;
    group: string;
    lints: Lint[];
}

/**
 * A struct that represents two character indices in a string: a start and an end.
 */
export declare class Span {
    private constructor();
    free(): void;
    len(): number;
    static new(start: number, end: number): Span;
    to_json(): string;
    is_empty(): boolean;
    static from_json(json: string): Span;
    start: number;
    end: number;
}

/**
 * A suggestion to fix a Lint.
 */
export declare class Suggestion {
    private constructor();
    free(): void;
    /**
     * Get the text that is going to replace the problematic section.
     * If [`Self::kind`] is `SuggestionKind::Remove`, this will return an empty
     * string.
     */
    get_replacement_text(): string;
    kind(): SuggestionKind;
    to_json(): string;
    static from_json(json: string): Suggestion;
}

/**
 * Tags the variant of suggestion.
 */
export declare enum SuggestionKind {
    /**
     * Replace the problematic text.
     */
    Replace = 0,
    /**
     * Remove the problematic text.
     */
    Remove = 1,
    /**
     * Insert additional text after the error.
     */
    InsertAfter = 2,
}

/**
 * Represents the summary of linting results and history.
 * Useful to show linting statistics or insights to the user.
 */
export declare interface Summary {
    /**
     * An object mapping each lint type to its count.
     * Example: `{ "Spelling": 4, "Capitalization": 1 }`
     */
    lint_counts: Record<string, number>;
    /**
     * The total number of fixes applied.
     */
    total_applied: number;
    /**
     * An object mapping misspelled words to their occurrence counts.
     * Example: `{ "mispelled": 1, "mispell": 1, "thigs": 2 }`
     */
    misspelled: Record<string, number>;
}

declare class SuperBinaryModule extends BinaryModule {
    createLinter(dialect?: Dialect): Promise<Linter_2>;
    getBinaryModule(): Promise<any>;
}

/** A Linter that spins up a dedicated web worker to do processing on a separate thread.
 * Main benefit: this Linter will not block the event loop for large documents.
 *
 * NOTE: This class will not work properly in Node. In that case, just use `LocalLinter`. */
export declare class WorkerLinter implements Linter {
    private binary;
    private serializer;
    private dialect?;
    private worker;
    private requestQueue;
    private working;
    constructor(init: LinterInit);
    private setupMainEventListeners;
    setup(): Promise<void>;
    lint(text: string, options?: LintOptions): Promise<Lint[]>;
    organizedLints(text: string, options?: LintOptions): Promise<Record<string, Lint[]>>;
    applySuggestion(text: string, lint: Lint, suggestion: Suggestion): Promise<string>;
    isLikelyEnglish(text: string): Promise<boolean>;
    isolateEnglish(text: string): Promise<string>;
    getLintConfig(): Promise<LintConfig>;
    setLintConfig(config: LintConfig): Promise<void>;
    getLintConfigAsJSON(): Promise<string>;
    setLintConfigWithJSON(config: string): Promise<void>;
    toTitleCase(text: string): Promise<string>;
    getLintDescriptionsAsJSON(): Promise<string>;
    getLintDescriptions(): Promise<Record<string, string>>;
    getLintDescriptionsHTMLAsJSON(): Promise<string>;
    getLintDescriptionsHTML(): Promise<Record<string, string>>;
    getDefaultLintConfigAsJSON(): Promise<string>;
    getDefaultLintConfig(): Promise<LintConfig>;
    ignoreLint(source: string, lint: Lint): Promise<void>;
    ignoreLintHash(hash: bigint): Promise<void>;
    exportIgnoredLints(): Promise<string>;
    importIgnoredLints(json: string): Promise<void>;
    contextHash(source: string, lint: Lint): Promise<bigint>;
    clearIgnoredLints(): Promise<void>;
    clearWords(): Promise<void>;
    importWords(words: string[]): Promise<void>;
    exportWords(): Promise<string[]>;
    getDialect(): Promise<Dialect>;
    setDialect(dialect: Dialect): Promise<void>;
    summarizeStats(start?: bigint, end?: bigint): Promise<any>;
    generateStatsFile(): Promise<string>;
    importStatsFile(statsFile: string): Promise<void>;
    /** Run a procedure on the remote worker. */
    private rpc;
    private submitRemainingRequests;
}

export { }
