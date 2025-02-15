import { toArray } from 'lodash-es';
import logoSvg from '../logo.svg';
import { Plugin, Menu, PluginManifest, App, Notice } from 'obsidian';
import { LintConfig, Linter, Suggestion } from 'harper.js';
import { LocalLinter, SuggestionKind, WorkerLinter, binary } from 'harper.js';
import { linter } from './lint';
import { Extension } from '@codemirror/state';
import { HarperSettingTab } from './HarperSettingTab';

function suggestionToLabel(sug: Suggestion) {
	if (sug.kind() == SuggestionKind.Remove) {
		return 'Remove';
	} else if (sug.kind() == SuggestionKind.Replace) {
		return `Replace with "${sug.get_replacement_text()}"`;
	} else if (sug.kind() == SuggestionKind.InsertAfter) {
		return `Insert "${sug.get_replacement_text()}" after this.`;
	}
}

export type Settings = {
	ignoredLints?: string;
	useWebWorker: boolean;
	lintSettings: LintConfig;
};

export default class HarperPlugin extends Plugin {
	private harper: Linter;
	private editorExtensions: Extension[];

	constructor(app: App, manifest: PluginManifest) {
		super(app, manifest);
		this.harper = new WorkerLinter({ binary });
		this.editorExtensions = [];
	}

	public async initializeFromSettings(settings: Settings | null) {
		if (settings == null) {
			settings = { useWebWorker: true, lintSettings: {} };
		}

		const oldSettings = await this.getSettings();

		if (settings.useWebWorker != oldSettings.useWebWorker) {
			if (settings.useWebWorker) {
				this.harper = new WorkerLinter({ binary });
			} else {
				this.harper = new LocalLinter({ binary });
			}
		} else {
			await this.harper.clearIgnoredLints();
		}

		if (settings.ignoredLints !== undefined) {
			await this.harper.importIgnoredLints(settings.ignoredLints);
		}

		await this.harper.setLintConfig(settings.lintSettings);
		this.harper.setup();

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
			lintSettings: await this.harper.getLintConfig()
		};
	}

	async onload() {
		if (typeof Response == 'undefined') {
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

	private setupStatusBar() {
		/** @type HTMLElement */
		const statusBarItem = this.addStatusBarItem();
		statusBarItem.className += ' mod-clickable';

		const button = document.createElement('span');
		button.style = 'width:24px';
		button.innerHTML = logoSvg;

		button.addEventListener('click', (event) => {
			const menu = new Menu();

			menu.addItem((item) =>
				item
					.setTitle(`${this.hasEditorLinter() ? 'Disable' : 'Enable'} automatic checking`)
					.setIcon('documents')
					.onClick(() => {
						this.toggleAutoLint();
					})
			);

			menu.showAtMouseEvent(event);
		});

		statusBarItem.appendChild(button);
	}

	private setupCommands() {
		this.addCommand({
			id: 'harper-toggle-auto-lint',
			name: 'Toggle automatic grammar checking',
			callback: () => this.toggleAutoLint()
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
		return this.editorExtensions.length != 0;
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

					return {
						from: span.start,
						to: span.end,
						severity: 'error',
						title: lint.lint_kind(),
						message: lint.message(),
						ignore: async () => {
							await this.harper.ignoreLint(lint);
							await this.reinitialize();
						},
						actions: lint.suggestions().map((sug) => {
							return {
								name: suggestionToLabel(sug),
								apply: (view) => {
									if (sug.kind() === SuggestionKind.Remove) {
										view.dispatch({
											changes: {
												from: span.start,
												to: span.end,
												insert: ''
											}
										});
									} else if (sug.kind() === SuggestionKind.Replace) {
										view.dispatch({
											changes: {
												from: span.start,
												to: span.end,
												insert: sug.get_replacement_text()
											}
										});
									} else if (sug.kind() === SuggestionKind.InsertAfter) {
										view.dispatch({
											changes: {
												from: span.end,
												to: span.end,
												insert: sug.get_replacement_text()
											}
										});
									}
								}
							};
						})
					};
				});
			},
			{
				delay: -1
			}
		);
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
