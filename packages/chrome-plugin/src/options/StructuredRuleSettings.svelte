<script lang="ts">
import { Select } from 'components';
import type { LintConfig, StructuredLintSetting } from 'harper.js';
import { startCase } from 'lodash-es';

type StructuredOneOfManySetting = Extract<
	StructuredLintSetting,
	{ OneOfMany: unknown }
>['OneOfMany'];

type RenderNode = GroupRenderNode | BoolRenderNode | OneOfManyRenderNode;

type GroupRenderNode = {
	kind: 'group';
	label: string;
	groupKey: string;
	indent: number;
	ruleNames: string[];
	ruleCount: number;
	state: 'default' | 'enable' | 'disable' | 'mixed';
	expanded: boolean;
	childNodes: RenderNode[];
};

type BoolRenderNode = {
	kind: 'bool';
	name: string;
	label: string;
	description: string;
	title: string;
	value: string;
	indent: number;
};

type OneOfManyRenderNode = {
	kind: 'oneOfMany';
	name: string;
	title: string;
	options: { value: string; label: string }[];
	value: string;
	setting: StructuredOneOfManySetting;
	indent: number;
};

export let settings: StructuredLintSetting[] = [];
export let nodes: RenderNode[] | undefined = undefined;
export let lintConfig: LintConfig = {};
export let lintDescriptions: Record<string, string> = {};
export let searchQueryLower = '';
export let expandedGroups: Record<string, boolean> = {};
export let groupPath: string[] = [];
export let indent = 0;
export let forceShow = false;
export let handleLintConfigChange: (next: LintConfig) => void;
export let handleToggleGroup: (groupKey: string) => void;

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

function displayRuleLabel(ruleName: string, label?: string | null): string {
	return label ?? startCase(ruleName);
}

function matchesRule(ruleName: string, label?: string | null, forceMatch = false): boolean {
	if (forceMatch || searchQueryLower === '') {
		return true;
	}

	const description = lintDescriptions[ruleName] ?? '';
	const displayLabel = displayRuleLabel(ruleName, label);
	return (
		ruleName.toLowerCase().includes(searchQueryLower) ||
		displayLabel.toLowerCase().includes(searchQueryLower) ||
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

	handleLintConfigChange(nextConfig);
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

	handleLintConfigChange(nextConfig);
}

function buildRenderNodes(
	settings: StructuredLintSetting[],
	path: string[],
	forceMatch: boolean,
	currentIndent: number,
): { nodes: RenderNode[]; ruleNames: string[] } {
	const out: RenderNode[] = [];
	const ruleNames: string[] = [];

	for (const setting of settings) {
		if ('Bool' in setting) {
			ruleNames.push(setting.Bool.name);

			if (!matchesRule(setting.Bool.name, setting.Bool.label, forceMatch)) {
				continue;
			}

			const label = displayRuleLabel(setting.Bool.name, setting.Bool.label);
			out.push({
				kind: 'bool',
				name: setting.Bool.name,
				label,
				description: lintDescriptions[setting.Bool.name] ?? '',
				title: `Set ${label} to its default, on, or off state.`,
				value: configValueToString(lintConfig[setting.Bool.name]),
				indent: currentIndent,
			});
			continue;
		}

		if ('OneOfMany' in setting) {
			ruleNames.push(...setting.OneOfMany.names);

			if (!settingVisible(setting, forceMatch)) {
				continue;
			}

			const name = setting.OneOfMany.name ?? setting.OneOfMany.labels?.join(' / ') ?? 'Choose one';
			out.push({
				kind: 'oneOfMany',
				name,
				title: `Choose an option for ${name}.`,
				options: [
					{ value: 'default', label: '⚙️ Default' },
					...setting.OneOfMany.names.map((value, index) => ({
						value,
						label: setting.OneOfMany.labels?.[index] ?? value,
					})),
				],
				value: oneOfManyValue(setting.OneOfMany),
				setting: setting.OneOfMany,
				indent: currentIndent,
			});
			continue;
		}

		const groupKey = [...path, setting.Group.label].join(' / ');
		const groupMatches =
			searchQueryLower !== '' && setting.Group.label.toLowerCase().includes(searchQueryLower);
		const child = buildRenderNodes(
			setting.Group.child.settings,
			[...path, setting.Group.label],
			forceMatch || groupMatches,
			currentIndent + 1,
		);
		ruleNames.push(...child.ruleNames);

		const visible = forceMatch || searchQueryLower === '' || groupMatches || child.nodes.length > 0;
		if (!visible) {
			continue;
		}

		out.push({
			kind: 'group',
			label: setting.Group.label,
			groupKey,
			indent: currentIndent,
			ruleNames: child.ruleNames,
			ruleCount: child.ruleNames.length,
			state: groupState(child.ruleNames),
			expanded:
				Boolean(expandedGroups[groupKey]) ||
				(searchQueryLower !== '' && (groupMatches || child.nodes.length > 0)),
			childNodes: child.nodes,
		});
	}

	return { nodes: out, ruleNames };
}

let renderedNodes: RenderNode[] = [];

$: {
	void searchQueryLower;
	void expandedGroups;
	void lintConfig;
	void lintDescriptions;
	renderedNodes = nodes ?? buildRenderNodes(settings, groupPath, forceShow, indent).nodes;
}
</script>

<div class="space-y-4">
	{#each renderedNodes as node}
		{#if node.kind === 'group'}
				<div class="space-y-3">
					<div class="flex items-start justify-between gap-4" style={rowStyle(node.indent)}>
						<div class="space-y-0.5">
							<h3 class="text-sm">{node.label}</h3>
							<p class="text-xs text-gray-600 dark:text-gray-400">{node.ruleCount} rules</p>
						</div>
						<div class="flex items-center gap-2">
							<button
								type="button"
								class="cursor-pointer inline-flex items-center gap-2 justify-center rounded-lg border border-gray-200 bg-white px-3 py-2 text-sm font-medium text-center text-gray-900 transition-colors hover:bg-gray-100 focus:outline-none focus:ring-4 focus:ring-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700"
								title={node.expanded
									? `Collapse the ${node.label} category`
									: `Expand the ${node.label} category`}
								onclick={() => handleToggleGroup(node.groupKey)}
							>
								{node.expanded ? 'Collapse' : 'Expand'}
							</button>
							<Select
								size="md"
								title={`Set all rules in the ${node.label} category to their default, on, or off state.`}
								value={node.state === 'mixed' ? 'default' : node.state}
								onchange={(event) => updateGroup(node.ruleNames, (event.target as HTMLSelectElement).value)}
							>
								<option value="default">{node.state === 'mixed' ? '⚙️ Default (mixed)' : '⚙️ Default'}</option>
								<option value="enable">✅ On</option>
								<option value="disable">🚫 Off</option>
							</Select>
						</div>
					</div>

					{#if node.expanded}
						<svelte:self
							nodes={node.childNodes}
							settings={[]}
							{lintConfig}
							{lintDescriptions}
							{searchQueryLower}
							{expandedGroups}
							groupPath={[]}
							indent={indent}
							forceShow={forceShow}
							{handleLintConfigChange}
							{handleToggleGroup}
						/>
					{/if}
				</div>
		{:else if node.kind === 'bool'}
				<div class="flex items-start justify-between gap-4" style={rowStyle(node.indent)}>
					<div class="space-y-0.5">
						<h3 class="text-sm">{node.label}</h3>
						<p class="text-xs">{@html node.description}</p>
					</div>
					<Select
						size="md"
						title={node.title}
						value={node.value}
						onchange={(event) => {
							const nextConfig: LintConfig = { ...lintConfig };
							nextConfig[node.name] = configStringToValue(
								(event.target as HTMLSelectElement).value,
							);
							handleLintConfigChange(nextConfig);
						}}
					>
						<option value="default">⚙️ Default</option>
						<option value="enable">✅ On</option>
						<option value="disable">🚫 Off</option>
					</Select>
				</div>
		{:else if node.kind === 'oneOfMany'}
			<div class="flex items-start justify-between gap-4" style={rowStyle(node.indent)}>
				<div class="space-y-0.5">
					<h3 class="text-sm">{node.name}</h3>
				</div>
				<Select
					size="md"
					title={node.title}
					value={node.value}
					onchange={(event) => updateOneOfMany(node.setting, (event.target as HTMLSelectElement).value)}
				>
					{#each node.options as option}
						<option value={option.value}>{option.label}</option>
					{/each}
				</Select>
			</div>
		{/if}
	{/each}
</div>
