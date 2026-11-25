/**
 * The table on /docs/rules, generated from the linter itself so it cannot fall out of date.
 */

import { block, escapeHtml } from './blocks.js';

/** Ask harper.js for every rule's description and whether it is on by default. */
export async function loadRules() {
	const { LocalLinter } = await import('harper.js');
	const { slimBinaryInlined } = await import('harper.js/slimBinaryInlined');
	const linter = new LocalLinter({ binary: slimBinaryInlined });

	try {
		const [descriptions, defaultConfig] = await Promise.all([
			linter.getLintDescriptionsHTML(),
			linter.getDefaultLintConfig(),
		]);

		return { descriptions, defaultConfig };
	} finally {
		await linter.dispose();
	}
}

/**
 * @param {{ descriptions: Record<string, string>, defaultConfig: Record<string, boolean | null> }} rules
 */
export function rulesTable({ descriptions, defaultConfig }) {
	const rows = Object.entries(descriptions).map(([name, description]) => {
		// Descriptions arrive as rendered Markdown. A table cell wants the text without the wrapper.
		const text = description.replace(/<\/?p\s*\/?>/g, '').trim();

		return `<tr><td>${escapeHtml(name)}</td><td>${defaultConfig[name] ? '✔️' : '❌'}</td><td>${text}</td></tr>`;
	});

	return block(
		'table',
		{ hasFixedLayout: false, className: 'harper-docs-rules' },
		`<figure class="wp-block-table harper-docs-rules"><table><thead><tr><th>Name</th><th>Enabled by Default</th><th>Description</th></tr></thead><tbody>${rows.join('')}</tbody></table></figure>`,
	);
}
