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
	description: string;
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

function configStringToValue(str: string): boolean | null {
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
					{ value: 'default', label: 'Default' },
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
			searchQueryLower !== '' &&
			(setting.Group.label.toLowerCase().includes(searchQueryLower) ||
				setting.Group.description.toLowerCase().includes(searchQueryLower));
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
			description: setting.Group.description,
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

function idFor(prefix: string, key: string): string {
	return `${prefix}-${key.replace(/[^a-zA-Z0-9]+/g, '-')}`;
}

const selectClass =
	'h-9 w-36 shrink-0 rounded-md border border-gray-200 bg-white px-3 text-xs font-semibold text-gray-900 dark:border-white/15! dark:bg-white/5! dark:text-white outline-none! transition-colors focus:border-primary! focus:ring-0! focus-visible:ring-2! focus-visible:ring-primary!';
const dividerClass = 'border-t border-gray-100 pt-3 dark:border-white/10!';
</script>

<div class="space-y-2">
    {#if nodes === undefined}
        <!-- Search result announcements (top-level instance only) -->
        <p class="sr-only" role="status" aria-live="polite" aria-atomic="true">
            {#if searchQueryLower !== ''}
                {renderedNodes.length === 0
                    ? 'No categories match your search.'
                    : `${renderedNodes.length} ${renderedNodes.length === 1 ? 'category matches' : 'categories match'} your search.`}
            {/if}
        </p>
        {#if renderedNodes.length === 0}
            <p class="py-8 text-center text-sm text-gray-600 dark:text-[#d1d5db]!">No rules match your search.</p>
        {/if}
    {/if}

    {#each renderedNodes as node}
        {#if node.kind === 'group'}
            {@const labelId = idFor('grp-label', node.groupKey)}
            {@const descId = idFor('grp-desc', node.groupKey)}
            {@const panelId = idFor('grp-panel', node.groupKey)}
            <!-- Top-level groups are cards; nested groups are flat rows -->
            <div
                class={node.indent === 0
                    ? 'rounded-lg border border-gray-200 bg-gray-50/60 p-3 transition-colors motion-reduce:transition-none dark:border-white/10! dark:bg-white/[0.03]! dark:hover:bg-white/[0.05]!'
                    : dividerClass}
            >
                <div class="flex items-start justify-between gap-4" style={rowStyle(node.indent)}>
                    <!-- Disclosure button: the whole title area toggles the group -->
                    <button
                        type="button"
                        class="group flex min-w-0 flex-1 cursor-pointer items-start gap-2 rounded-md text-left outline-none! focus-visible:ring-2! focus-visible:ring-primary!"
                        aria-expanded={node.expanded}
                        aria-controls={panelId}
                        aria-labelledby={labelId}
                        aria-describedby={descId}
                        onclick={() => handleToggleGroup(node.groupKey)}
                    >
                        <svg
                            class="mt-0.5 h-4 w-4 shrink-0 text-gray-400 transition-transform duration-150 motion-reduce:transition-none group-hover:text-primary dark:text-[#d1d5db]! dark:group-hover:text-primary! {node.expanded ? 'rotate-90' : ''}"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                            aria-hidden="true"
                            focusable="false"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M7.21 14.77a.75.75 0 0 1 .02-1.06L11.168 10 7.23 6.29a.75.75 0 1 1 1.04-1.08l4.5 4.25a.75.75 0 0 1 0 1.08l-4.5 4.25a.75.75 0 0 1-1.06-.02Z"
                                clip-rule="evenodd"
                            />
                        </svg>
                        <span class="min-w-0 space-y-0.5">
                            <span class="flex items-center gap-2">
                                <span id={labelId} class="text-sm font-bold text-gray-900 group-hover:text-primary dark:text-white">{node.label}</span>
                                <span aria-hidden="true" class="rounded-full bg-gray-200 px-2 py-0.5 font-mono text-[11px] font-semibold text-gray-700 dark:bg-primary/20! dark:text-primary!">{node.ruleCount}</span>
                            </span>
                            <span id={descId} class="block text-xs text-gray-600 dark:text-[#d1d5db]!">
                                {node.description}<span class="sr-only"> ({node.ruleCount} rules)</span>
                            </span>
                        </span>
                    </button>

                    <Select
                        size="sm"
                        title={`Set all rules in the ${node.label} category to their default, on, or off state.`}
                        aria-label={`${node.label}: set all ${node.ruleCount} rules`}
                        value={node.state === 'mixed' ? 'default' : node.state}
                        class={selectClass}
                        onchange={(event: Event) => updateGroup(node.ruleNames, (event.target as HTMLSelectElement).value)}
                    >
                        <option value="default">{node.state === 'mixed' ? 'Default (Mixed)' : 'Default'}</option>
                        <option value="enable">On</option>
                        <option value="disable">Off</option>
                    </Select>
                </div>

                {#if node.expanded}
                    <div id={panelId} role="group" aria-labelledby={labelId} class="mt-3 space-y-3">
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
                    </div>
                {/if}
            </div>
        {:else if node.kind === 'bool'}
            {@const labelId = idFor('rule-label', node.name)}
            {@const descId = idFor('rule-desc', node.name)}
            <div class="flex items-start justify-between gap-4 {dividerClass}" style={rowStyle(node.indent)}>
                <div class="min-w-0 space-y-0.5">
                    <h3 id={labelId} class="text-sm font-bold text-gray-900 dark:text-white">{node.label}</h3>
                    <p id={descId} class="text-xs text-gray-600 dark:text-[#d1d5db]!">{@html node.description}</p>
                </div>
                <Select
                    size="sm"
                    title={node.title}
                    aria-labelledby={labelId}
                    aria-describedby={descId}
                    value={node.value}
                    class={selectClass}
                    onchange={(event: Event) => {
                        const nextConfig: LintConfig = { ...lintConfig };
                        nextConfig[node.name] = configStringToValue(
                            (event.target as HTMLSelectElement).value,
                        );
                        handleLintConfigChange(nextConfig);
                    }}
                >
                    <option value="default">Default</option>
                    <option value="enable">On</option>
                    <option value="disable">Off</option>
                </Select>
            </div>
        {:else if node.kind === 'oneOfMany'}
            {@const labelId = idFor('choice-label', node.name)}
            <div class="flex items-start justify-between gap-4 {dividerClass}" style={rowStyle(node.indent)}>
                <div class="min-w-0 space-y-0.5">
                    <h3 id={labelId} class="text-sm font-bold text-gray-900 dark:text-white">{node.name}</h3>
                </div>
                <Select
                    size="sm"
                    title={node.title}
                    aria-labelledby={labelId}
                    value={node.value}
                    class={selectClass}
                    onchange={(event: Event) => updateOneOfMany(node.setting, (event.target as HTMLSelectElement).value)}
                >
                    {#each node.options as option}
                        <option value={option.value}>{option.label}</option>
                    {/each}
                </Select>
            </div>
        {/if}
    {/each}
</div>