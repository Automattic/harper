import './index.js';
import { Dialect } from 'harper.js';
import { startCase } from 'lodash-es';
import { type App, PluginSettingTab, Setting } from 'obsidian';
import type HarperPlugin from './index.js';
import type State from './State.js';
import type { Settings } from './State.js';
import { linesToString, stringToLines } from './textUtils';

export class HarperSettingTab extends PluginSettingTab {
	private state: State;
	private settings: Settings;
	private descriptionsHTML: Record<string, string>;
	private plugin: HarperPlugin;

	constructor(app: App, plugin: HarperPlugin, state: State) {
		super(app, plugin);
		this.state = state;
		this.plugin = plugin;

		// Poll every so often
		const update = () => {
			this.updateDescriptions();
			this.updateSettings();
			setTimeout(update, 1000);
		};

		update();
	}

	updateSettings() {
		this.state.getSettings().then((v) => {
			this.settings = v;
		});
	}

	updateDescriptions() {
		this.state.getDescriptionHTML().then((v) => {
			this.descriptionsHTML = v;
		});
	}

	display() {
		const { containerEl } = this;
		containerEl.empty();

		new Setting(containerEl)
			.setName('Use Web Worker')
			.setDesc(
				'Whether to run the Harper engine in a separate thread. Improves stability and speed at the cost of memory.',
			)
			.addToggle((toggle) =>
				toggle.setValue(this.settings.useWebWorker).onChange(async (value) => {
					this.settings.useWebWorker = value;
					await this.state.initializeFromSettings(this.settings);
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
					await this.state.initializeFromSettings(this.settings);
					this.plugin.updateStatusBar(dialect);
				});
		});

		new Setting(containerEl)
			.setName('Personal Dictionary')
			.setDesc(
				'Make edits to your personal dictionary. Add names, places, or terms you use often. Each line should contain its own word.',
			)
			.addTextArea((ta) => {
				ta.inputEl.cols = 20;
				ta.setValue(linesToString(this.settings.userDictionary ?? [''])).onChange(async (v) => {
					const dict = stringToLines(v);
					this.settings.userDictionary = dict;
					await this.state.initializeFromSettings(this.settings);
				});
			});

		new Setting(containerEl)
			.setName('Ignored Files')
			.setDesc(
				'Instruct Harper to ignore certain files in your vault. Accepts glob matches (`folder/**`, etc.)',
			)
			.addTextArea((ta) => {
				ta.inputEl.cols = 20;
				ta.setValue(linesToString(this.settings.ignoredGlobs ?? [''])).onChange(async (v) => {
					const lines = stringToLines(v);
					this.settings.ignoredGlobs = lines;
					await this.state.initializeFromSettings(this.settings);
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
						await this.state.initializeFromSettings(this.settings);
					});
			});

		new Setting(containerEl).setName('Get Notified of Harper Updates').addToggle((toggle) => {
			toggle.setValue(this.settings.showUpdateNotification ?? true).onChange(async (value) => {
				this.settings.showUpdateNotification = value;
				await this.state.initializeFromSettings(this.settings);
			});
		});

		new Setting(containerEl).setName('The Danger Zone').addButton((button) => {
			button
				.setButtonText('Forget Ignored Suggestions')
				.onClick(() => {
					this.settings.ignoredLints = undefined;
					this.state.initializeFromSettings(this.settings);
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

		const queryLower = searchQuery.toLowerCase();

		for (const setting of Object.keys(this.settings.lintSettings)) {
			const value = this.settings.lintSettings[setting];
			const descriptionHTML = this.descriptionsHTML[setting];

			if (
				searchQuery !== '' &&
				!(
					descriptionHTML.toLowerCase().contains(queryLower) ||
					setting.toLowerCase().contains(queryLower)
				)
			) {
				continue;
			}

			const fragment = document.createDocumentFragment();
			const template = document.createElement('template');
			template.innerHTML = descriptionHTML;
			fragment.appendChild(template.content);

			new Setting(containerEl)
				.setName(startCase(setting))
				.setDesc(fragment)
				.addDropdown((dropdown) =>
					dropdown
						.addOption('default', 'Default')
						.addOption('enable', 'On')
						.addOption('disable', 'Off')
						.setValue(valueToString(value))
						.onChange(async (value) => {
							this.settings.lintSettings[setting] = stringToValue(value);
							await this.state.initializeFromSettings(this.settings);
						}),
				);
		}
	}
}

function valueToString(value: boolean | null): string {
	switch (value) {
		case true:
			return 'enable';
		case false:
			return 'disable';
		case null:
			return 'default';
	}
}

function stringToValue(str: string): boolean | null {
	switch (str) {
		case 'enable':
			return true;
		case 'disable':
			return false;
		case 'default':
			return null;
	}

	throw 'Fell through case';
}
