import type { Extension } from '@codemirror/state';
import type { LintConfig, Linter, Suggestion } from 'harper.js';
import {
	Dialect,
	LocalLinter,
	SuggestionKind,
	WorkerLinter,
	binary,
	binaryInlined,
} from 'harper.js';
import { toArray } from 'lodash-es';
import { type App, Menu, Notice, Plugin, type PluginManifest } from 'obsidian';
import logoSvg from '../logo.svg';
import { HarperSettingTab } from './HarperSettingTab';
import { linter } from './lint';

function suggestionToLabel(sug: Suggestion) {
	if (sug.kind() === SuggestionKind.Remove) {
		return 'Remove';
	} else if (sug.kind() === SuggestionKind.Replace) {
		return `Replace with “${sug.get_replacement_text()}”`;
	} else if (sug.kind() === SuggestionKind.InsertAfter) {
		return `Insert “${sug.get_replacement_text()}” after this.`;
	}
}

const DEFAULT_DELAY = -1;

export type Settings = {
	ignoredLints?: string;
	useWebWorker: boolean;
	dialect?: Dialect;
	lintSettings: LintConfig;
	userDictionary?: string[];
	delay?: number;
};

export default class HarperPlugin extends Plugin {
	private harper: Linter;
	private editorExtensions: Extension[];
	private delay: number;
	private dialectSpan: HTMLSpanElement | null = null;

	constructor(app: App, manifest: PluginManifest) {
		super(app, manifest);
		this.harper = new WorkerLinter({ binary: binaryInlined });
		this.editorExtensions = [];
		this.delay = DEFAULT_DELAY;
	}

	public async initializeFromSettings(settings: Settings | null) {
		if (settings == null) {
			settings = { useWebWorker: true, lintSettings: {} };
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

		// Reinitialize it.
		if (this.hasEditorLinter()) {
			this.disableEditorLinter();
			this.enableEditorLinter();
		}

		await this.saveData(settings);
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
		};
	}

	async onload() {
		if (typeof Response === 'undefined') {
			new Notice('Please update your Electron version before running Harper.', 0);
			return;
		}

		const data = await this.loadData();
		await this.initializeFromSettings(data);
		this.registerEditorExtension(this.editorExtensions);
		this.setupCommands();
		this.setupStatusBar();
		this.enableEditorLinter();

		this.addSettingTab(new HarperSettingTab(this.app, this));
	}

	public async getDescriptions(): Promise<Record<string, string>> {
		return await this.harper.getLintDescriptions();
	}

	private getDialectStatus(dialectNum: Dialect): string {
		const code = {
			American: 'US',
			British: 'GB',
			Australian: 'AU',
			Canadian: 'CA',
		}[Dialect[dialectNum]];
		if (code === undefined) {
			return '';
		}
		return `${code
			.split('')
			.map((c) => String.fromCodePoint(c.charCodeAt(0) + 127397))
			.join('')}${code}`;
	}

	private setupStatusBar() {
		/** @type HTMLElement */
		const statusBarItem = this.addStatusBarItem();
		statusBarItem.className += ' mod-clickable';

		const button = document.createElement('span');
		button.style.display = 'flex';
		button.style.alignItems = 'center';

		const logo = document.createElement('span');
		logo.style.width = '24px';
		logo.innerHTML = logoSvg;
		button.appendChild(logo);

		const dialect = document.createElement('span');
		this.dialectSpan = dialect;

		this.harper.getDialect().then((dialectNum) => {
			this.updateStatusBar(dialectNum);
			button.appendChild(dialect);
		});

		button.addEventListener('click', (event) => {
			const menu = new Menu();

			menu.addItem((item) =>
				item
					.setTitle(`${this.hasEditorLinter() ? 'Disable' : 'Enable'} automatic checking`)
					.setIcon('documents')
					.onClick(() => {
						this.toggleAutoLint();
					}),
			);

			menu.showAtMouseEvent(event);
		});

		statusBarItem.appendChild(button);
	}

	private setupCommands() {
		this.addCommand({
			id: 'harper-toggle-auto-lint',
			name: 'Toggle automatic grammar checking',
			callback: () => this.toggleAutoLint(),
		});
	}

	enableEditorLinter() {
		if (!this.hasEditorLinter()) {
			this.editorExtensions.push(this.constructEditorLinter());
			this.app.workspace.updateOptions();
			console.log('Enabled');
		}
	}

	disableEditorLinter() {
		while (this.hasEditorLinter()) {
			this.editorExtensions.pop();
		}
		this.app.workspace.updateOptions();
	}

	hasEditorLinter(): boolean {
		return this.editorExtensions.length !== 0;
	}

	private toggleAutoLint() {
		if (this.hasEditorLinter()) {
			this.disableEditorLinter();
		} else {
			this.enableEditorLinter();
		}
	}

	/** Construct the linter plugin that actually shows the errors. */
	private constructEditorLinter(): Extension {
		return linter(
			async (view) => {
				const text = view.state.doc.sliceString(-1);
				const chars = toArray(text);

				const lints = await this.harper.lint(text);

				return lints.map((lint) => {
					const span = lint.span();

					span.start = charIndexToCodePointIndex(span.start, chars);
					span.end = charIndexToCodePointIndex(span.end, chars);

					const actions = lint.suggestions().map((sug) => {
						return {
							name: suggestionToLabel(sug),
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
							name: `Add “${word}” to your dictionary`,
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
						title: lint.lint_kind(),
						message: lint.message(),
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

	updateStatusBar(dialect: Dialect) {
		if (this.dialectSpan != null) {
			this.dialectSpan.innerHTML = this.getDialectStatus(dialect);
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
