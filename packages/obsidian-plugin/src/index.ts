import { Dialect } from 'harper.js';
import { type App, editorViewField, Menu, Notice, Plugin, type PluginManifest } from 'obsidian';
import logoSvg from '../logo.svg?raw';
import packageJson from '../package.json';
import { HarperSettingTab } from './HarperSettingTab';
import State from './State';

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

export async function logVersionInfo(showNotification: boolean): Promise<void> {
	try {
		const latest = await getLatestVersion();
		console.info(`Latest available Harper version: ${latest}`);

		if (latest != `v${packageJson.version}` && showNotification) {
			setTimeout(() => {
				new Notice('A newer version of Harper is available.');
			}, 5000);
		}
	} catch (err) {
		console.error(`Unable to obtain latest version: ${err}`);
	}

	console.info(`Current version: ${packageJson.version}`);
}

export default class HarperPlugin extends Plugin {
	private state: State;
	private dialectSpan: HTMLSpanElement | null = null;

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
		this.state.enableEditorLinter();

		this.addSettingTab(new HarperSettingTab(this.app, this, this.state));

		await logVersionInfo((await this.state.getSettings()).showUpdateNotification ?? true);
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
			callback: () => this.state.toggleAutoLint(),
		});
	}

	public updateStatusBar(dialect: Dialect) {
		if (this.dialectSpan != null) {
			this.dialectSpan.innerHTML = this.getDialectStatus(dialect);
		}
	}
}
