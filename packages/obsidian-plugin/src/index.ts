import { Dialect } from 'harper.js';
import { type App, editorInfoField, Menu, Notice, Plugin, type PluginManifest } from 'obsidian';
import logoSvg from '../logo.svg?raw';
import logoSvgDisabled from '../logo-disabled.svg?raw';
import { HarperSettingTab } from './HarperSettingTab';
import State from './State';

export default class HarperPlugin extends Plugin {
	private state: State;
	private dialectSpan: HTMLSpanElement | null = null;
	private logo: HTMLSpanElement | null = null;

	constructor(app: App, manifest: PluginManifest) {
		super(app, manifest);
		this.state = new State(
			(n) => this.saveData(n),
			() => this.app.workspace.updateOptions(),
			editorInfoField,
		);
	}

	async onload() {
		if (typeof Response === 'undefined') {
			new Notice('Please update your Electron version before running Harper.', 0);
			return;
		}

		const data = await this.loadData();
		await this.state.initializeFromSettings(data);
		this.registerEditorExtension(this.state.getCMEditorExtensions());
		this.setupCommands();
		this.setupStatusBar();
		if (!(data?.lintEnabled ?? true)) {
			this.state.disableEditorLinter();
		} else this.state.enableEditorLinter();

		this.addSettingTab(new HarperSettingTab(this.app, this, this.state));
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
