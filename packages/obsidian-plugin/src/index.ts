import type { Extension } from '@codemirror/state';
import { Dialect } from 'harper.js';
import { type App, Menu, Notice, Plugin, type PluginManifest } from 'obsidian';
import logoSvg from '../logo.svg?raw';
import packageJson from '../package.json';
import { HarperSettingTab } from './HarperSettingTab';
import Logic from './Logic';

async function getLatestVersion(): Promise<string> {
	const response = await fetch('https://writewithharper.com/latestversion', {
		headers: {
			'Harper-Version': packageJson.version,
		},
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return response.text();
}

export async function logVersionInfo(): Promise<void> {
	try {
		const latest = await getLatestVersion();
		console.info(`Latest available Harper version: ${latest}`);
	} catch (err) {
		console.error(`Unable to obtain latest version: ${err}`);
	}

	console.info(`Current version: ${packageJson.version}`);
}

logVersionInfo();

export default class HarperPlugin extends Plugin {
	private logic: Logic;
	private dialectSpan: HTMLSpanElement | null = null;

	constructor(app: App, manifest: PluginManifest) {
		super(app, manifest);
		this.logic = new Logic(
			(n) => this.saveData(n),
			() => this.app.workspace.updateOptions(),
		);
	}

	async onload() {
		if (typeof Response === 'undefined') {
			new Notice('Please update your Electron version before running Harper.', 0);
			return;
		}

		const data = await this.loadData();
		await this.logic.initializeFromSettings(data);
		this.registerEditorExtension(this.logic.getCMEditorExtensions());
		this.setupCommands();
		this.setupStatusBar();
		this.logic.enableEditorLinter();

		this.addSettingTab(new HarperSettingTab(this.app, this, this.logic));
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
		logo.innerHTML = logoSvg;
		button.appendChild(logo);

		const dialect = document.createElement('span');
		this.dialectSpan = dialect;

		this.logic.getSettings().then((settings) => {
			const dialectNum = settings.dialect ?? Dialect.American;
			this.updateStatusBar(dialectNum);
			button.appendChild(dialect);
		});

		button.addEventListener('click', (event) => {
			const menu = new Menu();

			menu.addItem((item) =>
				item
					.setTitle(`${this.logic.hasEditorLinter() ? 'Disable' : 'Enable'} automatic checking`)
					.setIcon('documents')
					.onClick(() => {
						this.logic.toggleAutoLint();
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
			callback: () => this.logic.toggleAutoLint(),
		});
	}

	public updateStatusBar(dialect: Dialect) {
		if (this.dialectSpan != null) {
			this.dialectSpan.innerHTML = this.getDialectStatus(dialect);
		}
	}
}
