import './index.js';
import { Dialect } from 'harper.js';
import { startCase } from 'lodash-es';
import { type App, PluginSettingTab, Setting } from 'obsidian';
import type HarperPlugin from './index.js';
import type { Settings } from './index.js';

export class HarperSettingTab extends PluginSettingTab {
	private plugin: HarperPlugin;
	private settings: Settings;
	private descriptions: Record<string, string>;

	constructor(app: App, plugin: HarperPlugin) {
		super(app, plugin);
		this.plugin = plugin;

		this.updateDescriptions();
		this.updateSettings();
	}

	updateSettings() {
		this.plugin.getSettings().then((v) => {
			this.settings = v;
		});
	}

	updateDescriptions() {
		this.plugin.getDescriptions().then((v) => {
			this.descriptions = v;
		});
	}

	display() {
		const { containerEl } = this;
		containerEl.empty();

		new Setting(containerEl).setName('Use Web Worker').addToggle((toggle) =>
			toggle.setValue(this.settings.useWebWorker).onChange(async (value) => {
				this.settings.useWebWorker = value;
				await this.plugin.initializeFromSettings(this.settings);
			}),
		);

		new Setting(containerEl).setName('English Dialect').addDropdown((dropdown) => {
			dropdown
				.addOption(Dialect.American.toString(), 'American')
				.addOption(Dialect.Canadian.toString(), 'Canadian')
				.addOption(Dialect.British.toString(), 'British')
				.addOption(Dialect.Australian.toString(), 'Australian')
				.setValue((this.settings.dialect ?? Dialect.American).toString())
				.onChange(async (value) => {
					const dialect = Number.parseInt(value);
					this.settings.dialect = dialect;
					await this.plugin.initializeFromSettings(this.settings);
					this.plugin.updateStatusBar(dialect);
				});
		});

		new Setting(containerEl)
			.setName('Delay')
			.setDesc(
				'Set the delay (in milliseconds) before Harper checks your work after you make a change. Set to -1 for no delay.',
			)
			.addSlider((slider) => {
				slider
					.setDynamicTooltip()
					.setLimits(-1, 10000, 50)
					.setValue(this.settings.delay ?? -1)
					.onChange(async (value) => {
						this.settings.delay = value;
						await this.plugin.initializeFromSettings(this.settings);
					});
			});

		new Setting(containerEl).setName('The Danger Zone').addButton((button) => {
			button
				.setButtonText('Forget Ignored Suggestions')
				.onClick(() => {
					this.settings.ignoredLints = undefined;
					this.plugin.initializeFromSettings(this.settings);
				})
				.setWarning();
		});

		new Setting(containerEl)
			.setName('Rules')
			.setDesc('Search for a specific Harper rule.')
			.addSearch((search) => {
				search.setPlaceholder('Search for a rule...').onChange((query) => {
					this.renderLintSettingsToId(query, 'HarperLintSettings');
				});
			});

		const lintSettings = document.createElement('DIV');
		lintSettings.id = 'HarperLintSettings';
		containerEl.appendChild(lintSettings);

		this.renderLintSettings('', lintSettings);
	}

	renderLintSettingsToId(searchQuery: string, id: string) {
		const el = document.getElementById(id);
		this.renderLintSettings(searchQuery, el!);
	}

	renderLintSettings(searchQuery: string, containerEl: HTMLElement) {
		containerEl.innerHTML = '';

		for (const setting of Object.keys(this.settings.lintSettings)) {
			const value = this.settings.lintSettings[setting];
			const description = this.descriptions[setting];

			if (
				searchQuery !== '' &&
				!(description.contains(searchQuery) || setting.contains(searchQuery))
			) {
				continue;
			}

			new Setting(containerEl)
				.setName(startCase(setting))
				.setDesc(description)
				.addDropdown((dropdown) =>
					dropdown
						.addOption('default', 'Default')
						.addOption('enable', 'On')
						.addOption('disable', 'Off')
						.setValue(valueToString(value))
						.onChange(async (value) => {
							this.settings.lintSettings[setting] = stringToValue(value);
							await this.plugin.initializeFromSettings(this.settings);
						}),
				);
		}
	}
}

function valueToString(value: boolean | undefined): string {
	switch (value) {
		case true:
			return 'enable';
		case false:
			return 'disable';
		case null:
			return 'default';
	}

	throw 'Fell through case';
}

function stringToValue(str: string): boolean | undefined {
	switch (str) {
		case 'enable':
			return true;
		case 'disable':
			return false;
		case 'default':
			return undefined;
	}

	throw 'Fell through case';
}
