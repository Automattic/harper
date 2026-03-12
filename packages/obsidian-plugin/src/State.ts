import type { Extension, StateField } from '@codemirror/state';
import type { Lint, LintConfig, Linter, Suggestion } from 'harper.js';
import { binaryInlined, type Dialect, LocalLinter, SuggestionKind, WorkerLinter } from 'harper.js';
import { minimatch } from 'minimatch';
import type { MarkdownFileInfo, Workspace } from 'obsidian';
import { linter } from './lint';

export type Settings = {
	ignoredLints?: string;
	useWebWorker: boolean;
	dialect?: Dialect;
	lintSettings: LintConfig;
	userDictionary?: string[];
	delay?: number;
	ignoredGlobs?: string[];
	lintEnabled?: boolean;
	regexMask?: string;
};

const DEFAULT_DELAY = -1;

/** The centralized state for the entire Obsidian plugin.
 * Since it also contains most business logic, for testing purpose it should not interact with Obsidian directly.*/
export default class State {
	private harper: Linter;
	private saveData: (data: any) => Promise<void>;
	private delay: number;
	private workspace: Workspace;
	private onExtensionChange: () => void;
	private ignoredGlobs?: string[];
	private editorInfoField?: StateField<MarkdownFileInfo>;
	private lintEnabled?: boolean;
	private regexMask?: string;

	/** The CodeMirror extension objects that should be inserted by the host. */
	private editorExtensions: Extension[];

	/** @param saveDataCallback A callback which will be used to save data on disk.
	 * @param onExtensionChange A callback this class will run when the extension array is modified.
	 * @param editorViewField Needed to provide support for ignoring files based on path.*/
	constructor(
		saveDataCallback: (data: any) => Promise<void>,
		onExtensionChange: () => void,
		_editorInfoField?: StateField<MarkdownFileInfo>,
	) {
		this.harper = new WorkerLinter({ binary: binaryInlined });
		this.delay = DEFAULT_DELAY;
		this.saveData = saveDataCallback;
		this.onExtensionChange = onExtensionChange;
		this.editorExtensions = [];

		this.editorInfoField = _editorInfoField;
	}

	public async initializeFromSettings(settings: Settings | null) {
		if (settings == null) {
			settings = {
				useWebWorker: true,
				lintEnabled: true,
				lintSettings: {},
			};
		}

		const defaultConfig = await this.harper.getDefaultLintConfig();
		for (const key of Object.keys(defaultConfig)) {
			if (settings.lintSettings[key] == undefined) {
				settings.lintSettings[key] = null;
			}
		}

		const oldSettings = await this.getSettings();

		if (
			settings.useWebWorker !== oldSettings.useWebWorker ||
			settings.dialect !== oldSettings.dialect
		) {
			if (settings.useWebWorker) {
				this.harper.dispose();
				this.harper = new WorkerLinter({ binary: binaryInlined, dialect: settings.dialect });
			} else {
				this.harper.dispose();
				this.harper = new LocalLinter({ binary: binaryInlined, dialect: settings.dialect });
			}
		} else {
			await this.harper.clearIgnoredLints();
		}

		if (settings.ignoredLints !== undefined) {
			await this.harper.importIgnoredLints(settings.ignoredLints);
		}

		if (settings.userDictionary != null) {
			await this.harper.clearWords();
			if (settings.userDictionary.length > 0) {
				await this.harper.importWords(settings.userDictionary);
			}
		}

		await this.harper.setLintConfig(settings.lintSettings);
		this.harper.setup();

		this.delay = settings.delay ?? DEFAULT_DELAY;
		this.ignoredGlobs = settings.ignoredGlobs;
		this.lintEnabled = settings.lintEnabled;
		this.regexMask = settings.regexMask;

		// Reinitialize it.
		if (this.hasEditorLinter()) {
			this.disableEditorLinter(false);
			this.enableEditorLinter(false);
		}

		await this.saveData(settings);
	}

	/** Construct the linter plugin that actually shows the errors. */
	private constructEditorLinter(): Extension {
		return linter(
			async (view) => {
				const ignoredGlobs = this.ignoredGlobs ?? [];

				if (this.editorInfoField != null) {
					const mdView = view.state.field(this.editorInfoField, false);
					const file = mdView?.file;

					if (file != null) {
						const path = file.path;
						for (const glob of ignoredGlobs) {
							if (minimatch(path, glob)) {
								return [];
							}
						}
					}
				}

				const text = view.state.doc.sliceString(-1);
				const lints = await this.harper.organizedLints(text, { regex_mask: this.regexMask });

				return Object.entries(lints).flatMap(([linterName, lints]) =>
					lints.map((lint) => {
						const span = lint.span();

						const actions = lint.suggestions().map((sug) => {
							return {
								kind: 'suggestion' as const,
								name:
									sug.kind() == SuggestionKind.Replace
										? sug.get_replacement_text()
										: suggestionToLabel(sug),
								title: suggestionToLabel(sug),
								apply: (view, from, to) => {
									if (sug.kind() === SuggestionKind.Remove) {
										view.dispatch({
											changes: {
												from,
												to,
												insert: '',
											},
											selection: {
												anchor: from,
											},
										});
									} else if (sug.kind() === SuggestionKind.Replace) {
										const replacement = sug.get_replacement_text();
										view.dispatch({
											changes: {
												from,
												to,
												insert: replacement,
											},
											selection: {
												anchor: from + replacement.length,
											},
										});
									} else if (sug.kind() === SuggestionKind.InsertAfter) {
										const replacement = sug.get_replacement_text();
										view.dispatch({
											changes: {
												from: to,
												to,
												insert: replacement,
											},
											selection: {
												anchor: to + replacement.length,
											},
										});
									}
								},
							};
						});

						if (lint.lint_kind() === 'Spelling') {
							const word = lint.get_problem_text();

							actions.push({
								kind: 'dictionary',
								name: '📖',
								title: `Add “${word}” to your dictionary`,
								apply: (view, _from, to) => {
									view.dispatch({
										selection: {
											anchor: to,
										},
									});
									this.harper.importWords([word]);
									this.reinitialize();
								},
							});
						}

						return {
							from: span.start,
							to: span.end,
							source: linterName,
							severity: 'error',
							title: lint.lint_kind_pretty(),
							renderMessage: (_view) => {
								const node = document.createElement('template');
								node.innerHTML = lint.message_html();
								return node.content;
							},
							ignore: async () => {
								await this.ignoreLints(text, [lint]);
							},
							disable: async () => {
								const lintConfig = await this.harper.getLintConfig();
								lintConfig[linterName] = false;
								await this.harper.setLintConfig(lintConfig);

								await this.reinitialize();
							},

							actions,
						};
					}),
				);
			},
			{
				delay: this.delay,
			},
		);
	}

	/** Use this method instead of interacting with the linter directly. */
	public async ignoreLints(text: string, lints: Lint[]) {
		for (const lint of lints) {
			await this.harper.ignoreLint(text, lint);
		}

		await this.reinitialize();
	}

	public async reinitialize() {
		const settings = await this.getSettings();
		await this.initializeFromSettings(settings);
	}

	public async getSettings(): Promise<Settings> {
		const usingWebWorker = this.harper instanceof WorkerLinter;

		const userDictionary = await this.harper.exportWords();
		userDictionary.sort();

		return {
			ignoredLints: await this.harper.exportIgnoredLints(),
			useWebWorker: usingWebWorker,
			lintSettings: await this.harper.getLintConfig(),
			userDictionary,
			dialect: await this.harper.getDialect(),
			delay: this.delay,
			ignoredGlobs: this.ignoredGlobs,
			lintEnabled: this.lintEnabled,
			regexMask: this.regexMask,
		};
	}

	/**
	 * Reset all lint rule overrides back to their defaults (null).
	 * Persists and reinitializes state to apply changes.
	 */
	public async resetAllRulesToDefaults(): Promise<void> {
		const settings = await this.getSettings();
		for (const key of Object.keys(settings.lintSettings)) {
			settings.lintSettings[key] = null;
		}
		await this.initializeFromSettings(settings);
	}

	/**
	 * Enable or disable all lint rules in bulk by setting explicit values.
	 * This overrides individual rule settings until changed again.
	 */
	public async setAllRulesEnabled(enabled: boolean): Promise<void> {
		const settings = await this.getSettings();
		for (const key of Object.keys(settings.lintSettings)) {
			settings.lintSettings[key] = enabled;
		}
		await this.initializeFromSettings(settings);
	}

	public async getDescriptionHTML(): Promise<Record<string, string>> {
		return await this.harper.getLintDescriptionsHTML();
	}

	/** Expose the default lint configuration for UI rendering. */
	public async getDefaultLintConfig(): Promise<LintConfig> {
		return await this.harper.getDefaultLintConfig();
	}

	/** Effective config: merges defaults with overrides (null/undefined uses default). */
	public async getEffectiveLintConfig(): Promise<Record<string, boolean>> {
		const defaults = (await this.getDefaultLintConfig()) as Record<string, boolean>;
		const overrides = (await this.getSettings()).lintSettings as Record<
			string,
			boolean | null | undefined
		>;
		const effective: Record<string, boolean> = {};
		for (const key of Object.keys(defaults)) {
			const v = overrides[key];
			effective[key] = v === null || v === undefined ? defaults[key] : Boolean(v);
		}
		return effective;
	}

	/** Determine if any rules are effectively enabled, considering defaults. */
	public async areAnyRulesEnabled(): Promise<boolean> {
		const settings = await this.getSettings();
		const defaults = await this.getDefaultLintConfig();
		for (const key of Object.keys(settings.lintSettings)) {
			const v = settings.lintSettings[key] as boolean | null | undefined;
			const def = (defaults as Record<string, boolean | undefined>)[key];
			const effective = v === null || v === undefined ? def : v;
			if (effective) return true;
		}
		return false;
	}

	/** Get a reference to the CM editor extensions.
	 * Do not mutate the returned value, except via methods on this class. */
	public getCMEditorExtensions(): Extension[] {
		return this.editorExtensions;
	}

	/** Enables the editor linter by adding an extension to the editor extensions array. */
	public enableEditorLinter(reinit = true) {
		if (!this.hasEditorLinter()) {
			this.editorExtensions.push(this.constructEditorLinter());
			this.lintEnabled = true;
			this.onExtensionChange();
			if (reinit) this.reinitialize();
			console.log('Enabled');
		}
	}

	/** Disables the editor linter by removing the extension from the editor extensions array. */
	public disableEditorLinter(reinit = true) {
		while (this.hasEditorLinter()) {
			this.editorExtensions.pop();
		}
		this.lintEnabled = false;
		this.onExtensionChange();
		if (reinit) this.reinitialize();
		console.log('Disabled');
	}

	public hasEditorLinter(): boolean {
		return this.editorExtensions.length !== 0;
	}

	public toggleAutoLint() {
		if (this.hasEditorLinter()) {
			this.disableEditorLinter();
		} else {
			this.enableEditorLinter();
		}
	}

	/** Get a reference to the current linter.
	 * It's best not to hold on to this type and to instead use this function again if another reference is needed. */
	public getLinter(): Linter {
		return this.harper;
	}
}

function suggestionToLabel(sug: Suggestion) {
	if (sug.kind() === SuggestionKind.Remove) {
		return 'Remove';
	} else if (sug.kind() === SuggestionKind.Replace) {
		return `Replace with “${sug.get_replacement_text()}”`;
	} else if (sug.kind() === SuggestionKind.InsertAfter) {
		return `Insert “${sug.get_replacement_text()}” after this.`;
	}
}
