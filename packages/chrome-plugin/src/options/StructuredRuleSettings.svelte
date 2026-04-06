<script lang="ts">
import { Button, Select } from 'components';
import type { LintConfig, StructuredLintSetting } from 'harper.js';

type StructuredOneOfManySetting = Extract<
	StructuredLintSetting,
	{ OneOfMany: unknown }
>['OneOfMany'];

export let settings: StructuredLintSetting[] = [];
export let lintConfig: LintConfig = {};
export let lintDescriptions: Record<string, string> = {};
export let searchQueryLower = '';
export let expandedGroups: Record<string, boolean> = {};
export let groupPath: string[] = [];
export let indent = 0;
export let forceShow = false;
export let onLintConfigChange: (next: LintConfig) => void;
export let onToggleGroup: (groupKey: string) => void;

function configValueToString(value: boolean | undefined | null): string {
	switch (value) {
		case true:
			return 'enable';
		case false:
			return 'disable';
		case undefined:
		case null:
			return 'default';
	}
}

function configStringToValue(str: string): boolean | undefined | null {
	switch (str) {
		case 'enable':
			return true;
		case 'disable':
			return false;
		case 'default':
			return null;
	}

	throw new Error('Unexpected config value');
}

function rowStyle(indent: number): string | undefined {
	return indent > 0 ? `padding-left: ${indent * 1.5}rem` : undefined;
}

function matchesRule(ruleName: string, label?: string | null, forceMatch = false): boolean {
	if (forceMatch || searchQueryLower === '') {
		return true;
	}

	const description = lintDescriptions[ruleName] ?? '';
	return (
		ruleName.toLowerCase().includes(searchQueryLower) ||
		(label?.toLowerCase().includes(searchQueryLower) ?? false) ||
		description.toLowerCase().includes(searchQueryLower)
	);
}

function settingVisible(setting: StructuredLintSetting, forceMatch = false): boolean {
	if (forceMatch || searchQueryLower === '') {
		return true;
	}

	if ('Bool' in setting) {
		return matchesRule(setting.Bool.name, setting.Bool.label, false);
	}

	if ('OneOfMany' in setting) {
		return (
			(setting.OneOfMany.name?.toLowerCase().includes(searchQueryLower) ?? false) ||
			setting.OneOfMany.names.some((name, index) =>
				matchesRule(name, setting.OneOfMany.labels?.[index], false),
			)
		);
	}

	if (setting.Group.label.toLowerCase().includes(searchQueryLower)) {
		return true;
	}

	return setting.Group.child.settings.some((child) => settingVisible(child, false));
}

function collectRuleNames(settings: StructuredLintSetting[]): string[] {
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

		out.push(...collectRuleNames(setting.Group.child.settings));
	}

	return out;
}

function groupKeyFor(label: string): string {
	return [...groupPath, label].join(' / ');
}

function groupState(ruleNames: string[]): 'default' | 'enable' | 'disable' | 'mixed' {
	const values = ruleNames.map((name) => lintConfig[name] ?? null);

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

function updateGroup(ruleNames: string[], value: string) {
	const nextConfig: LintConfig = { ...lintConfig };

	for (const ruleName of ruleNames) {
		nextConfig[ruleName] = configStringToValue(value);
	}

	onLintConfigChange(nextConfig);
}

function oneOfManyValue(setting: StructuredOneOfManySetting): string {
	const values = setting.names.map((name) => lintConfig[name] ?? null);
	if (values.every((value) => value === null)) {
		return 'default';
	}

	return setting.names.find((name) => lintConfig[name] === true) ?? 'default';
}

function updateOneOfMany(setting: StructuredOneOfManySetting, selected: string) {
	const nextConfig: LintConfig = { ...lintConfig };

	for (const name of setting.names) {
		nextConfig[name] = selected === 'default' ? null : name === selected;
	}

	onLintConfigChange(nextConfig);
}
</script>

<div class="space-y-4">
	{#each settings as setting}
		{#if 'Group' in setting}
			{@const visible = settingVisible(setting, forceShow)}
			{#if visible}
				{@const groupKey = groupKeyFor(setting.Group.label)}
				{@const groupMatches = searchQueryLower !== '' && setting.Group.label.toLowerCase().includes(searchQueryLower)}
				{@const ruleNames = collectRuleNames(setting.Group.child.settings)}
				{@const state = groupState(ruleNames)}
				{@const expanded = Boolean(expandedGroups[groupKey]) || groupMatches || (searchQueryLower !== '' && !groupMatches)}

				<div class="space-y-3">
					<div class="flex items-start justify-between gap-4" style={rowStyle(indent)}>
						<div class="space-y-0.5">
							<h3 class="text-sm">{setting.Group.label}</h3>
							<p class="text-xs text-gray-600 dark:text-gray-400">{ruleNames.length} rules</p>
						</div>
						<div class="flex items-center gap-2">
							<Button
								size="sm"
								color="light"
								title={expanded
									? `Collapse the ${setting.Group.label} category`
									: `Expand the ${setting.Group.label} category`}
								on:click={() => onToggleGroup(groupKey)}
							>
								{expanded ? 'Collapse' : 'Expand'}
							</Button>
							<Select
								size="md"
								title={`Set all rules in the ${setting.Group.label} category to their default, on, or off state.`}
								value={state === 'mixed' ? 'default' : state}
								onchange={(event) => updateGroup(ruleNames, (event.target as HTMLSelectElement).value)}
							>
								<option value="default">{state === 'mixed' ? '⚙️ Default (mixed)' : '⚙️ Default'}</option>
								<option value="enable">✅ On</option>
								<option value="disable">🚫 Off</option>
							</Select>
						</div>
					</div>

					{#if expanded}
						<svelte:self
							settings={setting.Group.child.settings}
							{lintConfig}
							{lintDescriptions}
							{searchQueryLower}
							{expandedGroups}
							groupPath={[...groupPath, setting.Group.label]}
							indent={indent + 1}
							forceShow={forceShow || groupMatches}
							{onLintConfigChange}
							{onToggleGroup}
						/>
					{/if}
				</div>
			{/if}
		{:else if 'Bool' in setting}
			{#if matchesRule(setting.Bool.name, setting.Bool.label, forceShow)}
				<div class="flex items-start justify-between gap-4" style={rowStyle(indent)}>
					<div class="space-y-0.5">
						<h3 class="text-sm">{setting.Bool.label ?? setting.Bool.name}</h3>
						<p class="text-xs">{@html lintDescriptions[setting.Bool.name] ?? ''}</p>
					</div>
					<Select
						size="md"
						title={`Set ${setting.Bool.label ?? setting.Bool.name} to its default, on, or off state.`}
						value={configValueToString(lintConfig[setting.Bool.name])}
						onchange={(event) => {
							const nextConfig: LintConfig = { ...lintConfig };
							nextConfig[setting.Bool.name] = configStringToValue(
								(event.target as HTMLSelectElement).value,
							);
							onLintConfigChange(nextConfig);
						}}
					>
						<option value="default">⚙️ Default</option>
						<option value="enable">✅ On</option>
						<option value="disable">🚫 Off</option>
					</Select>
				</div>
			{/if}
		{:else if settingVisible(setting, forceShow)}
			<div class="flex items-start justify-between gap-4" style={rowStyle(indent)}>
				<div class="space-y-0.5">
					<h3 class="text-sm">{setting.OneOfMany.name ?? setting.OneOfMany.labels?.join(' / ') ?? 'Choose one'}</h3>
				</div>
				<Select
					size="md"
					title={`Choose an option for ${setting.OneOfMany.name ?? setting.OneOfMany.labels?.join(' / ') ?? 'this rule group'}.`}
					value={oneOfManyValue(setting.OneOfMany)}
					onchange={(event) => updateOneOfMany(setting.OneOfMany, (event.target as HTMLSelectElement).value)}
				>
					<option value="default">⚙️ Default</option>
					{#each setting.OneOfMany.names as name, index}
						<option value={name}>{setting.OneOfMany.labels?.[index] ?? name}</option>
					{/each}
				</Select>
			</div>
		{/if}
	{/each}
</div>
