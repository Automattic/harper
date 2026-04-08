import './index.js';
import type { StructuredLintConfig, StructuredLintSetting } from 'harper.js';
import { Dialect } from 'harper.js';
import { startCase } from 'lodash-es';
import type { ButtonComponent } from 'obsidian';
import { type App, Notice, PluginSettingTab, Setting } from 'obsidian';
import type HarperPlugin from './index.js';
import type State from './State.js';
import type { Settings } from './State.js';
import { linesToString, stringToLines } from './textUtils';

const LintSettingId = 'HarperLintSettings';

export class HarperSettingTab extends PluginSettingTab {
	private settings?: Settings;
	private descriptionsHTML: Record<string, string> = {};
	private defaultLintConfig: Record<string, boolean> = {};
	private structuredLintConfig?: StructuredLintConfig;
	private currentRuleSearchQuery = '';
	private plugin: HarperPlugin;
	private toggleAllButton?: ButtonComponent;
	private expandedGroups = new Set<string>();

	private get state() {
		return this.plugin.state;
	}

	constructor(app: App, plugin: HarperPlugin) {
		super(app, plugin);
		this.plugin = plugin;
	}

	update() {
		this.updateDescriptions();
		this.updateSettings();
		this.updateDefaults();
		this.updateStructuredConfig();
	}

	updateSettings() {
		this.state.getSettings().then((v) => {
			const shouldRedrawWholeTab = this.settings == null;
			this.settings = v;
			this.updateToggleAllRulesButton();
			if (shouldRedrawWholeTab) {
				this.display(false);
				return;
			}
			this.rerenderLintSettings();
		});
	}

	updateDescriptions() {
		this.state.getDescriptionHTML().then((v) => {
			this.descriptionsHTML = v;
			this.rerenderLintSettings();
		});
	}

	updateDefaults() {
		this.state.getDefaultLintConfig().then((v) => {
			this.defaultLintConfig = v as unknown as Record<string, boolean>;
			this.updateToggleAllRulesButton();
			this.rerenderLintSettings();
		});
	}

	updateStructuredConfig() {
		this.state.getStructuredLintConfig().then((v) => {
			this.structuredLintConfig = v;
			this.rerenderLintSettings();
		});
	}

	display(update = true) {
		if (update) {
			this.update();
			this.display(false);
		}

		const { containerEl } = this;
		containerEl.empty();

		if (!this.settings) {
			const loading = document.createElement('p');
			loading.textContent = 'Loading Harper settings...';
			containerEl.appendChild(loading);
			return;
		}

		const settings = this.settings;

		new Setting(containerEl)
			.setName('Use Web Worker')
			.setDesc(
				'Whether to run the Harper engine in a separate thread. Improves stability and speed at the cost of memory.',
			)
			.addToggle((toggle) =>
				toggle.setValue(settings.useWebWorker).onChange(async (value) => {
					settings.useWebWorker = value;
					await this.state.initializeFromSettings(settings);
				}),
			);

		new Setting(containerEl).setName('English Dialect').addDropdown((dropdown) => {
			dropdown
				.addOption(Dialect.American.toString(), 'American')
				.addOption(Dialect.Canadian.toString(), 'Canadian')
				.addOption(Dialect.British.toString(), 'British')
				.addOption(Dialect.Australian.toString(), 'Australian')
				.addOption(Dialect.Indian.toString(), 'Indian')
				.setValue((settings.dialect ?? Dialect.American).toString())
				.onChange(async (value) => {
					const dialect = Number.parseInt(value, 10);
					settings.dialect = dialect;
					await this.state.initializeFromSettings(settings);
					this.plugin.updateStatusBar(dialect);
				});
		});

		new Setting(containerEl)
			.setName('Activate Harper')
			.setDesc('Enable or disable Harper with this option.')
			.addToggle((toggle) =>
				toggle.setValue(settings.lintEnabled ?? true).onChange(async (_value) => {
					this.state.toggleAutoLint();
					this.plugin.updateStatusBar();
				}),
			);

		new Setting(containerEl)
			.setName('Use Web-Style Lints')
			.setDesc(
				'Use a straight underline with a background color instead of the default squiggly underline.',
			)
			.addToggle((toggle) =>
				toggle.setValue(this.settings.useWebStyleLints ?? false).onChange(async (value) => {
					this.settings.useWebStyleLints = value;
					await this.state.initializeFromSettings(this.settings);
				}),
			);

		new Setting(containerEl)
			.setName('Mask')
			.setDesc(
				"Hide certain text from Harper's pedantic gaze with a regular expression. Follows the standard Rust syntax.",
			)
			.addTextArea((ta) =>
				ta.setValue(settings.regexMask ?? '').onChange(async (value) => {
					settings.regexMask = value;
					await this.state.initializeFromSettings(settings);
				}),
			);

		new Setting(containerEl)
			.setName('Personal Dictionary')
			.setDesc(
				'Make edits to your personal dictionary. Add names, places, or terms you use often. Each line should contain its own word.',
			)
			.addTextArea((ta) => {
				ta.inputEl.cols = 20;
				ta.setValue(linesToString(settings.userDictionary ?? [''])).onChange(async (v) => {
					const dict = stringToLines(v);
					settings.userDictionary = dict;
					await this.state.initializeFromSettings(settings);
				});
			});

		new Setting(containerEl)
			.setName('Ignored Files')
			.setDesc(
				'Instruct Harper to ignore certain files in your vault. Accepts glob matches (`folder/**`, etc.)',
			)
			.addTextArea((ta) => {
				ta.inputEl.cols = 20;
				ta.setValue(linesToString(settings.ignoredGlobs ?? [''])).onChange(async (v) => {
					const lines = stringToLines(v);
					settings.ignoredGlobs = lines;
					await this.state.initializeFromSettings(settings);
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
					.setValue(settings.delay ?? -1)
					.onChange(async (value) => {
						settings.delay = value;
						await this.state.initializeFromSettings(settings);
					});
			});

		new Setting(containerEl).setName('The Danger Zone').addButton((button) => {
			button
				.setButtonText('Forget Ignored Suggestions')
				.onClick(() => {
					settings.ignoredLints = undefined;
					this.state.initializeFromSettings(settings);
				})
				.setWarning();
		});

		new Setting(containerEl)
			.setName('Rules')
			.setDesc('Search for a specific Harper rule.')
			.addSearch((search) => {
				search.setPlaceholder('Search for a rule...').onChange((query) => {
					this.currentRuleSearchQuery = query;
					this.renderLintSettingsToId(query, LintSettingId);
				});
			});

		// Global reset for rule overrides
		new Setting(containerEl)
			.setName('Reset Rules to Defaults')
			.setDesc(
				'Restore all rule overrides back to their default values. This does not affect other settings.',
			)
			.addButton((button) => {
				button
					.setButtonText('Reset All to Defaults')
					.onClick(async () => {
						const confirmed = confirm(
							'Reset all rule overrides to their defaults? This cannot be undone.',
						);
						if (!confirmed) return;
						await this.state.resetAllRulesToDefaults();
						this.settings = await this.state.getSettings();
						this.renderLintSettingsToId(this.currentRuleSearchQuery, LintSettingId);
						this.updateToggleAllRulesButton();
						new Notice('Harper rules reset to defaults');
					})
					.setWarning();
			});

		// Single bulk toggle button: If any rules are enabled, turn all off; otherwise turn all on.
		new Setting(containerEl)
			.setName('Toggle All Rules')
			.setDesc(
				'Enable or disable all rules in bulk. Overrides individual rule settings until changed again.',
			)
			.addButton((button) => {
				this.toggleAllButton = button;
				this.updateToggleAllRulesButton();
				button.setWarning().onClick(async () => {
					const anyEnabledNow = await this.state.areAnyRulesEnabled();
					const action = anyEnabledNow ? 'Disable' : 'Enable';
					const confirmed = confirm(`${action} all rules? This will override individual settings.`);
					if (!confirmed) return;

					await this.state.setAllRulesEnabled(!anyEnabledNow);
					this.settings = await this.state.getSettings();
					this.renderLintSettingsToId(this.currentRuleSearchQuery, LintSettingId);
					this.updateToggleAllRulesButton();
					new Notice(`All Harper rules ${anyEnabledNow ? 'disabled' : 'enabled'}`);
				});
			});

		const lintSettings = document.createElement('DIV');
		lintSettings.id = LintSettingId;
		containerEl.appendChild(lintSettings);

		Promise.all([this.state.getDefaultLintConfig(), this.state.getStructuredLintConfig()]).then(
			([defaults, structured]) => {
				this.defaultLintConfig = defaults as unknown as Record<string, boolean>;
				this.structuredLintConfig = structured;
				this.renderLintSettingsToId(this.currentRuleSearchQuery, lintSettings.id);
			},
		);
	}

	private rerenderLintSettings() {
		this.renderLintSettingsToId(this.currentRuleSearchQuery, LintSettingId);
	}

	private async updateToggleAllRulesButton() {
		if (!this.toggleAllButton) return;
		const anyEnabled = await this.state.areAnyRulesEnabled();
		this.toggleAllButton.setButtonText(anyEnabled ? 'Disable All Rules' : 'Enable All Rules');
	}

	async renderLintSettingsToId(searchQuery: string, id: string) {
		const el = document.getElementById(id);
		if (!el) return;
		const effective = await this.state.getEffectiveLintConfig();
		this.renderLintSettings(searchQuery, el, effective);
	}

	private renderLintSettings(
		searchQuery: string,
		containerEl: HTMLElement,
		effectiveConfig: Record<string, boolean>,
	) {
		containerEl.innerHTML = '';

		const queryLower = searchQuery.toLowerCase();

		if (!this.structuredLintConfig) {
			return;
		}

		const rendered = this.renderStructuredSettings(
			this.structuredLintConfig.settings,
			containerEl,
			effectiveConfig,
			queryLower,
			0,
			false,
		);

		if (rendered === 0) {
			const empty = document.createElement('p');
			empty.textContent = 'No rules match your search.';
			containerEl.appendChild(empty);
		}
	}

	private renderStructuredSettings(
		settings: StructuredLintSetting[],
		containerEl: HTMLElement,
		effectiveConfig: Record<string, boolean>,
		queryLower: string,
		depth: number,
		forceShow: boolean,
		path: string[] = [],
	): number {
		let rendered = 0;

		for (const setting of settings) {
			if ('Group' in setting) {
				const groupPath = [...path, setting.Group.label];
				const groupKey = groupPath.join(' / ');
				const groupMatches =
					queryLower !== '' &&
					(setting.Group.label.toLowerCase().includes(queryLower) ||
						setting.Group.description.toLowerCase().includes(queryLower));

				const childEl = document.createElement('div');
				childEl.style.marginLeft = '1.5rem';

				const childCount = this.renderStructuredSettings(
					setting.Group.child.settings,
					childEl,
					effectiveConfig,
					queryLower,
					depth + 1,
					forceShow || groupMatches,
					groupPath,
				);

				if (childCount === 0) {
					continue;
				}

				const groupRules = this.collectRuleNames(setting.Group.child.settings);
				const isExpanded = queryLower !== '' || this.expandedGroups.has(groupKey);

				this.renderGroupSetting(
					setting.Group.label,
					setting.Group.description,
					groupKey,
					groupRules,
					containerEl,
					childEl,
					isExpanded,
				);
				rendered += childCount;
				continue;
			}

			if ('Bool' in setting) {
				if (!this.shouldRenderRule(setting.Bool.name, setting.Bool.label, queryLower, forceShow)) {
					continue;
				}

				this.renderBoolSetting(setting.Bool.name, setting.Bool.label, containerEl, effectiveConfig);
				rendered += 1;
				continue;
			}

			if ('OneOfMany' in setting) {
				if (!this.shouldRenderOneOfMany(setting.OneOfMany, queryLower, forceShow)) {
					continue;
				}

				this.renderOneOfManySetting(setting.OneOfMany, containerEl, effectiveConfig);
				rendered += 1;
			}
		}

		return rendered;
	}

	private collectRuleNames(settings: StructuredLintSetting[]): string[] {
		const out: string[] = [];

		for (const setting of settings) {
			if ('Bool' in setting) {
				out.push(setting.Bool.name);
				continue;
			}

			if ('OneOfMany' in setting) {
				out.push(...setting.OneOfMany.names);
				continue;
			}

			out.push(...this.collectRuleNames(setting.Group.child.settings));
		}

		return out;
	}

	private renderGroupSetting(
		label: string,
		description: string,
		groupKey: string,
		ruleNames: string[],
		containerEl: HTMLElement,
		childEl: HTMLElement,
		isExpanded: boolean,
	) {
		const state = this.getGroupState(ruleNames);
		const defaultLabel = state === 'mixed' ? 'Default (mixed)' : 'Default';

		new Setting(containerEl)
			.setName(label)
			.setDesc(description)
			.addButton((button) => {
				this.setButtonHoverText(
					button,
					isExpanded ? `Collapse the ${label} category` : `Expand the ${label} category`,
				);

				button.setButtonText(isExpanded ? 'Collapse' : 'Expand').onClick(() => {
					if (this.expandedGroups.has(groupKey)) {
						this.expandedGroups.delete(groupKey);
					} else {
						this.expandedGroups.add(groupKey);
					}

					this.rerenderLintSettings();
				});
			})
			.addDropdown((dropdown) => {
				this.setDropdownHoverText(
					dropdown,
					`Set all rules in the ${label} category to their default, on, or off state.`,
				);

				dropdown
					.addOption('default', defaultLabel)
					.addOption('enable', 'On')
					.addOption('disable', 'Off')
					.setValue(state === 'mixed' ? 'default' : state)
					.onChange(async (value) => {
						const settings = this.settings;
						if (!settings) {
							return;
						}

						for (const ruleName of ruleNames) {
							settings.lintSettings[ruleName] = value === 'default' ? null : value === 'enable';
						}

						await this.state.initializeFromSettings(settings);
						this.settings = await this.state.getSettings();
						this.rerenderLintSettings();
						this.updateToggleAllRulesButton();
					});
			});

		if (isExpanded) {
			containerEl.appendChild(childEl);
		}
	}

	private setButtonHoverText(button: ButtonComponent, text: string) {
		const maybeTooltipButton = button as ButtonComponent & {
			setTooltip?: (text: string) => ButtonComponent;
			buttonEl?: HTMLButtonElement;
		};

		maybeTooltipButton.setTooltip?.(text);
		maybeTooltipButton.buttonEl?.setAttribute('title', text);
	}

	private setDropdownHoverText(dropdown: { selectEl?: HTMLSelectElement }, text: string) {
		dropdown.selectEl?.setAttribute('title', text);
	}

	private getGroupState(ruleNames: string[]): 'default' | 'enable' | 'disable' | 'mixed' {
		const settings = this.settings;
		if (!settings) {
			return 'default';
		}

		const values = ruleNames.map((ruleName) => settings.lintSettings[ruleName] ?? null);

		if (values.every((value) => value === null)) {
			return 'default';
		}

		if (values.every((value) => value === true)) {
			return 'enable';
		}

		if (values.every((value) => value === false)) {
			return 'disable';
		}

		return 'mixed';
	}

	private shouldRenderRule(
		ruleName: string,
		label: string | null | undefined,
		queryLower: string,
		forceShow: boolean,
	) {
		if (forceShow || queryLower === '') {
			return true;
		}

		const descriptionHTML = this.descriptionsHTML?.[ruleName] ?? '';
		return (
			ruleName.toLowerCase().includes(queryLower) ||
			(label?.toLowerCase().includes(queryLower) ?? false) ||
			descriptionHTML.toLowerCase().includes(queryLower)
		);
	}

	private shouldRenderOneOfMany(
		setting: Extract<StructuredLintSetting, { OneOfMany: unknown }>['OneOfMany'],
		queryLower: string,
		forceShow: boolean,
	) {
		if (forceShow || queryLower === '') {
			return true;
		}

		return setting.names.some((name, index) =>
			this.shouldRenderRule(name, setting.labels?.[index], queryLower, false),
		);
	}

	private renderBoolSetting(
		setting: string,
		label: string | null | undefined,
		containerEl: HTMLElement,
		effectiveConfig: Record<string, boolean>,
	) {
		if (!this.settings) {
			return;
		}

		const value = this.settings.lintSettings[setting];
		const descriptionHTML = this.descriptionsHTML?.[setting] ?? '';
		const fragment = document.createDocumentFragment();
		if (descriptionHTML !== '') {
			const template = document.createElement('template');
			template.innerHTML = descriptionHTML;
			fragment.appendChild(template.content);
		}

		const defaultVal = this.defaultLintConfig?.[setting];

		new Setting(containerEl)
			.setName(label ?? startCase(setting))
			.setDesc(fragment)
			.addDropdown((dropdown) => {
				this.setDropdownHoverText(
					dropdown,
					`Set ${label ?? startCase(setting)} to on or off. Labels marked default reflect the current default behavior.`,
				);

				const effective: boolean | undefined = effectiveConfig[setting];
				const usingDefault = value === null;
				const onLabel = usingDefault && defaultVal === true ? 'On (default)' : 'On';
				const offLabel = usingDefault && defaultVal === false ? 'Off (default)' : 'Off';
				dropdown
					.addOption('enable', onLabel)
					.addOption('disable', offLabel)
					.setValue(effective ? 'enable' : 'disable')
					.onChange(async (v) => {
						// The structured config only organizes rules for display.
						// Persist changes through the flat lint config keyed by rule name.
						this.settings.lintSettings[setting] = v === 'enable';
						await this.state.initializeFromSettings(this.settings);
						this.settings = await this.state.getSettings();
						this.renderLintSettingsToId(this.currentRuleSearchQuery, LintSettingId);
						this.updateToggleAllRulesButton();
					});
			});
	}

	private renderOneOfManySetting(
		setting: Extract<StructuredLintSetting, { OneOfMany: unknown }>['OneOfMany'],
		containerEl: HTMLElement,
		effectiveConfig: Record<string, boolean>,
	) {
		if (!this.settings) {
			return;
		}

		const currentName =
			setting.names.find((name) => effectiveConfig[name]) ?? setting.name ?? setting.names[0];
		const label =
			setting.labels?.join(' / ') ?? setting.names.map((name) => startCase(name)).join(' / ');

		new Setting(containerEl).setName(label).addDropdown((dropdown) => {
			this.setDropdownHoverText(dropdown, `Choose an option for ${label}.`);

			for (const [index, name] of setting.names.entries()) {
				dropdown.addOption(name, setting.labels?.[index] ?? startCase(name));
			}

			dropdown.setValue(currentName).onChange(async (selected) => {
				// The structured config only organizes rules for display.
				// Persist changes through the flat lint config keyed by rule name.
				for (const name of setting.names) {
					this.settings.lintSettings[name] = name === selected;
				}

				await this.state.initializeFromSettings(this.settings);
				this.settings = await this.state.getSettings();
				this.renderLintSettingsToId(this.currentRuleSearchQuery, LintSettingId);
				this.updateToggleAllRulesButton();
			});
		});
	}
}

// Note: dropdowns present only On/Off. When using defaults (unset),
// the matching option label includes "(default)".
