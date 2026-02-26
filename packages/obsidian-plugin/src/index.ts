import type { EditorView } from '@codemirror/view';
import { Dialect } from 'harper.js';
import {
	type App,
	editorInfoField,
	MarkdownView,
	Menu,
	Notice,
	Plugin,
	type PluginManifest,
} from 'obsidian';
import logoSvg from '../logo.svg?raw';
import logoSvgDisabled from '../logo-disabled.svg?raw';
import { HarperSettingTab } from './HarperSettingTab';
import {
	addWordToDictionaryFromVisibleTooltip,
	applySuggestionFromVisibleTooltip,
	canAddWordToDictionaryFromVisibleTooltip,
	canApplySuggestionFromVisibleTooltip,
	canDismissFocusedLintTooltip,
	canIgnoreVisibleTooltipDiagnostic,
	canNavigateDiagnostics,
	dismissFocusedLintTooltip,
	ignoreVisibleTooltipDiagnostic,
	navigateDiagnostic,
} from './lint';
import State from './State';

export default class HarperPlugin extends Plugin {
	state: State;
	private dialectSpan: HTMLSpanElement | null = null;
	private logo: HTMLSpanElement | null = null;
	private settings: HarperSettingTab | null = null;

	constructor(app: App, manifest: PluginManifest) {
		super(app, manifest);
	}

	async onload() {
		if (typeof Response === 'undefined') {
			new Notice('Please update your Electron version before running Harper.', 0);
			return;
		}

		const data = await this.loadData();

		this.app.workspace.onLayoutReady(async () => {
			this.state = new State(
				(n) => this.saveData(n),
				() => this.app.workspace.updateOptions(),
				editorInfoField,
			);

			await this.state.initializeFromSettings(data);
			this.registerEditorExtension(this.state.getCMEditorExtensions());
			if (!(data?.lintEnabled ?? true)) {
				this.state.disableEditorLinter(false);
			} else this.state.enableEditorLinter(false);
			this.settings?.update();

			this.setupStatusBar();
		});

		this.settings = new HarperSettingTab(this.app, this);
		this.addSettingTab(this.settings);

		this.setupCommands();
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
		const statusBarItem: HTMLElement = this.addStatusBarItem();
		statusBarItem.className += ' mod-clickable';

		const button = document.createElement('span');
		button.style.display = 'flex';
		button.style.alignItems = 'center';

		const logo = document.createElement('span');
		logo.style.width = '24px';
		logo.innerHTML = this.state.hasEditorLinter() ? logoSvg : logoSvgDisabled;
		this.logo = logo;
		button.appendChild(logo);

		const dialect = document.createElement('span');
		this.dialectSpan = dialect;

		this.state.getSettings().then((settings) => {
			const dialectNum = settings.dialect ?? Dialect.American;
			this.updateStatusBar(dialectNum);
			button.appendChild(dialect);
		});

		button.addEventListener('click', (event) => {
			const menu = new Menu();

			menu.addItem((item) =>
				item
					.setTitle(`${this.state.hasEditorLinter() ? 'Disable' : 'Enable'} automatic checking`)
					.setIcon('documents')
					.onClick(() => {
						this.toggleAutoLint();
					}),
			);

			menu.addItem((item) =>
				item
					.setTitle('Ignore all errors in file')
					.setIcon('eraser')
					.onClick(() => {
						this.doIgnoreAllFlow();
					}),
			);

			menu.showAtMouseEvent(event);
		});

		statusBarItem.appendChild(button);
	}

	/** Preferred over directly calling `this.state.toggleAutoLint()` */
	private toggleAutoLint() {
		this.state.toggleAutoLint();
		this.updateStatusBar();
	}

	private setupCommands() {
		this.addCommand({
			id: 'harper-toggle-auto-lint',
			name: 'Toggle automatic grammar checking',
			callback: () => {
				this.toggleAutoLint();
			},
		});

		this.addCommand({
			id: 'harper-ignore-all-in-buffer',
			name: 'Ignore all errors in the open file',
			callback: async () => {
				await this.doIgnoreAllFlow();
			},
		});

		this.addCommand({
			id: 'harper-jump-to-next-suggestion',
			name: 'Jump to next suggestion',
			checkCallback: (checking) => {
				const editorView = this.getActiveEditorView();
				if (!editorView) return false;
				if (checking) return canNavigateDiagnostics(editorView);
				return navigateDiagnostic(editorView, 'next');
			},
		});

		this.addCommand({
			id: 'harper-jump-to-previous-suggestion',
			name: 'Jump to previous suggestion',
			checkCallback: (checking) => {
				const editorView = this.getActiveEditorView();
				if (!editorView) return false;
				if (checking) return canNavigateDiagnostics(editorView);
				return navigateDiagnostic(editorView, 'previous');
			},
		});

		this.addCommand(this.getApplySuggestionCommand(1));
		this.addCommand(this.getApplySuggestionCommand(2));
		this.addCommand(this.getApplySuggestionCommand(3));

		this.addCommand({
			id: 'harper-add-word-to-dictionary',
			name: 'Add current word to dictionary',
			checkCallback: (checking) => {
				const editorView = this.getActiveEditorView();
				if (!editorView) return false;
				if (checking) return canAddWordToDictionaryFromVisibleTooltip(editorView);
				return addWordToDictionaryFromVisibleTooltip(editorView);
			},
		});

		this.addCommand({
			id: 'harper-ignore-focused-diagnostic',
			name: 'Ignore focused diagnostic',
			hotkeys: [{ modifiers: ['Mod', 'Shift'], key: 'i' }],
			checkCallback: (checking) => {
				const editorView = this.getActiveEditorView();
				if (!editorView) return false;
				if (checking) return canIgnoreVisibleTooltipDiagnostic(editorView);
				return ignoreVisibleTooltipDiagnostic(editorView);
			},
		});

		this.addCommand({
			id: 'harper-dismiss-focused-tooltip',
			name: 'Dismiss focused suggestion tooltip',
			hotkeys: [{ modifiers: [], key: 'Escape' }],
			checkCallback: (checking) => {
				const editorView = this.getActiveEditorView();
				if (!editorView) return false;
				if (checking) return canDismissFocusedLintTooltip(editorView);
				return dismissFocusedLintTooltip(editorView);
			},
		});
	}

	private getApplySuggestionCommand(n: number) {
		return {
			id: `harper-apply-suggestion-${n}`,
			name: `Apply suggestion #${n}`,
			checkCallback: (checking: boolean) => {
				const editorView = this.getActiveEditorView();
				if (!editorView) return false;
				if (checking) return canApplySuggestionFromVisibleTooltip(editorView, n);
				return applySuggestionFromVisibleTooltip(editorView, n);
			},
		};
	}

	private getActiveEditorView(): EditorView | null {
		const view = this.app.workspace.getActiveViewOfType(MarkdownView);
		if (!view) return null;
		return (view.editor as any).cm as EditorView;
	}

	/** Trigger the flow for ignoring all files in a document, including a confirmation modal. */
	public async doIgnoreAllFlow() {
		const file = this.app.workspace.getActiveFile();
		if (file != null) {
			const text = await this.app.vault.read(file);

			const lints = await this.state.getLinter().lint(text);
			const confirmation = confirm(
				`Are you sure you want to ignore ${lints.length} errors from Harper?`,
			);

			if (confirmation) {
				await this.state.ignoreLints(text, lints);
			}
		} else {
			new Notice('No file currently open.');
		}
	}

	public updateStatusBar(dialect?: Dialect) {
		if (this.logo != null) {
			this.logo.innerHTML = this.state.hasEditorLinter() ? logoSvg : logoSvgDisabled;
		}
		if (typeof dialect !== 'undefined') {
			if (this.dialectSpan != null) {
				this.dialectSpan.innerHTML = this.getDialectStatus(dialect);
			}
		}
	}
}
