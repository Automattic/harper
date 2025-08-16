import type { Extension, StateField } from '@codemirror/state';
import type { LintConfig, Linter, Suggestion } from 'harper.js';
import { binaryInlined, type Dialect, LocalLinter, SuggestionKind, WorkerLinter } from 'harper.js';
import { toArray } from 'lodash-es';
import { minimatch } from 'minimatch';
import type { MarkdownFileInfo, MarkdownView, Workspace } from 'obsidian';
import { linter } from './lint';

export type Settings = {
	ignoredLints?: string;
	useWebWorker: boolean;
	dialect?: Dialect;
	lintSettings: LintConfig;
	userDictionary?: string[];
	delay?: number;
	ignoredGlobs?: string[];
	/** An undefined value should be treated the same as `true`. */
	showUpdateNotification?: boolean;
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
	private editorViewField?: StateField<MarkdownFileInfo>;
	private showUpdateNotification?: boolean;

	/** The CodeMirror extension objects that should be inserted by the host. */
	private editorExtensions: Extension[];

	/** @param saveDataCallback A callback which will be used to save data on disk.
	 * @param onExtensionChange A callback this class will run when the extension array is modified.
	 * @param editorViewField Needed to provide support for ignoring files based on path.*/
	constructor(
		saveDataCallback: (data: any) => Promise<void>,
		onExtensionChange: () => void,
		editorViewField?: StateField<MarkdownFileInfo>,
	) {
		this.harper = new WorkerLinter({ binary: binaryInlined });
		this.delay = DEFAULT_DELAY;
		this.saveData = saveDataCallback;
		this.onExtensionChange = onExtensionChange;
		this.editorExtensions = [];
	}

	public async initializeFromSettings(settings: Settings | null) {
		if (settings == null) {
			settings = { useWebWorker: true, lintSettings: {} };
		}

		const defaultConfig = await this.harper.getDefaultLintConfig();
		for (const [key, value] of Object.entries(defaultConfig)) {
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
				this.harper = new WorkerLinter({ binary: binaryInlined, dialect: settings.dialect });
			} else {
				this.harper = new LocalLinter({ binary: binaryInlined, dialect: settings.dialect });
			}
		} else {
			await this.harper.clearIgnoredLints();
		}

		if (settings.ignoredLints !== undefined) {
			await this.harper.importIgnoredLints(settings.ignoredLints);
		}

		if (settings.userDictionary != null && settings.userDictionary.length > 0) {
			await this.harper.importWords(settings.userDictionary);
		}

		await this.harper.setLintConfig(settings.lintSettings);
		this.harper.setup();

		this.delay = settings.delay ?? DEFAULT_DELAY;
		this.ignoredGlobs = settings.ignoredGlobs;
		this.showUpdateNotification = settings.showUpdateNotification;

		// Reinitialize it.
		if (this.hasEditorLinter()) {
			this.disableEditorLinter();
			this.enableEditorLinter();
		}

		await this.saveData(settings);
	}

	/** Construct the linter plugin that actually shows the errors. */
	private constructEditorLinter(): Extension {
		return linter(
			async (view) => {
				const ignoredGlobs = this.ignoredGlobs ?? [];

				if (this.editorViewField != null) {
					const mdView = view.state.field(this.editorViewField) as MarkdownView;
					const file = mdView?.file;
					const path = file?.path!;

					if (path != null) {
						for (const glob of ignoredGlobs) {
							if (minimatch(path, glob)) {
								return [];
							}
						}
					}
				}

				const text = view.state.doc.sliceString(-1);
				const chars = toArray(text);

				const lints = await this.harper.lint(text);

				return lints.map((lint) => {
					const span = lint.span();

					span.start = charIndexToCodePointIndex(span.start, chars);
					span.end = charIndexToCodePointIndex(span.end, chars);

					const actions = lint.suggestions().map((sug) => {
						return {
							name:
								sug.kind() == SuggestionKind.Replace
									? sug.get_replacement_text()
									: suggestionToLabel(sug),
							title: suggestionToLabel(sug),
							apply: (view) => {
								if (sug.kind() === SuggestionKind.Remove) {
									view.dispatch({
										changes: {
											from: span.start,
											to: span.end,
											insert: '',
										},
									});
								} else if (sug.kind() === SuggestionKind.Replace) {
									view.dispatch({
										changes: {
											from: span.start,
											to: span.end,
											insert: sug.get_replacement_text(),
										},
									});
								} else if (sug.kind() === SuggestionKind.InsertAfter) {
									view.dispatch({
										changes: {
											from: span.end,
											to: span.end,
											insert: sug.get_replacement_text(),
										},
									});
								}
							},
						};
					});

					if (lint.lint_kind() === 'Spelling') {
						const word = lint.get_problem_text();

						actions.push({
							name: '📖',
							title: `Add “${word}” to your dictionary`,
							apply: (view) => {
								this.harper.importWords([word]);
								this.reinitialize();
							},
						});
					}

					return {
						from: span.start,
						to: span.end,
						severity: 'error',
						title: lint.lint_kind_pretty(),
						renderMessage: (view) => {
							const node = document.createElement('template');
							node.innerHTML = lint.message_html();
							return node.content;
						},
						ignore: async () => {
							await this.harper.ignoreLint(text, lint);
							await this.reinitialize();
						},
						actions,
					};
				});
			},
			{
				delay: this.delay,
			},
		);
	}

	public async reinitialize() {
		const settings = await this.getSettings();
		await this.initializeFromSettings(settings);
	}

	public async getSettings(): Promise<Settings> {
		const usingWebWorker = this.harper instanceof WorkerLinter;

		return {
			ignoredLints: await this.harper.exportIgnoredLints(),
			useWebWorker: usingWebWorker,
			lintSettings: await this.harper.getLintConfig(),
			userDictionary: await this.harper.exportWords(),
			dialect: await this.harper.getDialect(),
			delay: this.delay,
			ignoredGlobs: this.ignoredGlobs,
			showUpdateNotification: this.showUpdateNotification,
		};
	}

	public async getDescriptionHTML(): Promise<Record<string, string>> {
		return await this.harper.getLintDescriptionsHTML();
	}

	/** Get a reference to the CM editor extensions.
	 * Do not mutate the returned value, except via methods on this class. */
	public getCMEditorExtensions(): Extension[] {
		return this.editorExtensions;
	}

	/** Enables the editor linter by adding an extension to the editor extensions array. */
	public enableEditorLinter() {
		if (!this.hasEditorLinter()) {
			this.editorExtensions.push(this.constructEditorLinter());
			this.onExtensionChange();
			console.log('Enabled');
		}
	}

	/** Disables the editor linter by removing the extension from the editor extensions array. */
	public disableEditorLinter() {
		while (this.hasEditorLinter()) {
			this.editorExtensions.pop();
		}
		this.onExtensionChange();
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
}

/** Harper returns positions based on char indexes,
 * but Obsidian identifies locations in documents based on Unicode code points.
 * This converts between from the former to the latter.*/
function charIndexToCodePointIndex(index: number, sourceChars: string[]): number {
	let traversed = 0;

	for (let i = 0; i < index; i++) {
		const delta = sourceChars[i].length;

		traversed += delta;
	}

	return traversed;
}

function suggestionToLabel(sug: Suggestion) {
	if (sug.kind() === SuggestionKind.Remove) {
		return 'Remove';
	} else if (sug.kind() === SuggestionKind.Replace) {
		return `“Replace with ${sug.get_replacement_text()}”`;
	} else if (sug.kind() === SuggestionKind.InsertAfter) {
		return `Insert “${sug.get_replacement_text()}” after this.`;
	}
}
