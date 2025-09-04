import { Dialect } from 'harper.js';
import { type App, editorViewField, Menu, Notice, Plugin, type PluginManifest } from 'obsidian';
import logoSvg from '../logo.svg?raw';
import logoSvgDisabled from '../logo-disabled.svg?raw';
import packageJson from '../package.json';
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
			editorViewField,
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
						this.state.toggleAutoLint();
						this.updateStatusBar();
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
			callback: () => {
				this.state.toggleAutoLint();
				this.updateStatusBar();
			},
		});
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
