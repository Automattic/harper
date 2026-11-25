/**
 * docs/sidebar.yml: the order and grouping of the docs, and the source of the section pages.
 */

import { parse as parseYaml } from 'yaml';
import { block, escapeHtml } from './blocks.js';

/**
 * @returns {{ title: string, description: string, entries: object[] }}
 */
export function readSidebar(source) {
	const data = parseYaml(source);

	if (typeof data?.title !== 'string' || !Array.isArray(data?.sidebar)) {
		throw new Error('docs/sidebar.yml needs a title and a sidebar list');
	}

	return { title: data.title, description: data.description ?? '', entries: data.sidebar };
}

const linksOf = (entry) => (Array.isArray(entry.items) ? entry.items : [entry]);
const urlOf = (link) => (link.page ? `/docs/${link.page}` : link.href);

/** Every page named in the sidebar, in reading order. */
export function sidebarPages(entries) {
	return entries.flatMap(linksOf).flatMap((link) => (link.page ? [link.page] : []));
}

/** The groups that get a page of their own at /docs/<section>. */
export function sidebarSections(entries) {
	return entries.filter((entry) => Array.isArray(entry.items) && entry.section);
}

/** The shape the site's theme reads: groups of { title, to } links, and bare links. */
export function themeSidebar(entries) {
	return entries.map((entry) =>
		Array.isArray(entry.items)
			? {
					...(entry.title ? { title: entry.title } : {}),
					items: entry.items.map((link) => ({ title: link.title, to: urlOf(link) })),
				}
			: { title: entry.title, to: urlOf(entry) },
	);
}

/** A list of links, each with the linked page's description under it. */
export function indexBlock(links, descriptions) {
	const items = links.map((link) => {
		const description = descriptions.get(link.to);
		const text = `<a href="${escapeHtml(link.to)}"><strong>${escapeHtml(link.title)}</strong></a>${
			description ? `<br />${escapeHtml(description)}` : ''
		}`;

		return block('list-item', null, `<li>${text}</li>`);
	});

	return block(
		'list',
		{ className: 'harper-docs-index' },
		`<ul class="wp-block-list harper-docs-index">${items.join('\n')}</ul>`,
	);
}

/** What /docs itself lists: the top-level pages and one link per section. */
export function rootLinks(entries) {
	return entries.flatMap((entry) => {
		if (!Array.isArray(entry.items)) {
			return [{ title: entry.title, to: urlOf(entry) }];
		}

		return entry.section
			? [{ title: entry.title, to: `/docs/${entry.section}` }]
			: entry.items.map((link) => ({ title: link.title, to: urlOf(link) }));
	});
}

export function sectionLinks(section) {
	return section.items.map((link) => ({ title: link.title, to: urlOf(link) }));
}
