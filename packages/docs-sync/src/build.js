/**
 * Build the docs bundle: every page under docs/ as WordPress content, the section pages, the
 * sidebar and the API reference, each with a hash so the site can tell what changed.
 */

import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync, realpathSync, statSync } from 'node:fs';
import path from 'node:path';
import { DocsProblem, parsePage, renderPage } from './markdown.js';
import { readReference } from './reference.js';
import { rulesTable } from './rules.js';
import {
	indexBlock,
	readSidebar,
	rootLinks,
	sectionLinks,
	sidebarPages,
	sidebarSections,
	themeSidebar,
} from './sidebar.js';

export const BUNDLE_FORMAT = 1;

const MAX_INCLUDE_BYTES = 256 * 1024;

/** Everything wrong with the docs, reported together so one run shows all of it. */
export class DocsBuildError extends Error {
	constructor(problems) {
		super(
			`${problems.length} problem(s) in the docs:\n${problems.map((p) => `  ${p}`).join('\n')}`,
		);
		this.problems = problems;
	}
}

/**
 * @param {object} options
 * @param {string} options.repo Absolute path of the repository checkout.
 * @param {string} [options.docsDir] Folder of Markdown pages, relative to the repo.
 * @param {string} [options.staticDir] Folder whose files the site serves from its root.
 * @param {string|null} [options.referenceDir] Output of packages/harper.js/docs.sh, or null to leave the reference out.
 * @param {object|null} [options.rules] From loadRules(), or null if no page shows the rules.
 * @param {{path: string, anchor: string}[]} [options.clientLinks] Deep links shipped clients open.
 * @param {string} [options.repository] owner/name, for links to source files.
 * @param {string} [options.branch]
 * @param {string} [options.commit]
 */
export function buildBundle(options) {
	const {
		repo,
		docsDir = 'docs',
		staticDir = 'packages/web/static',
		referenceDir = null,
		rules = null,
		clientLinks = [],
		repository = 'Automattic/harper',
		branch = 'master',
		commit = '',
	} = options;

	const root = realpathSync(repo);
	const docsRoot = path.join(root, docsDir);
	const problems = [];
	const attempt = (work) => {
		try {
			return work();
		} catch (error) {
			if (!(error instanceof DocsProblem)) {
				throw error;
			}
			problems.push(error.message);
			return null;
		}
	};

	const sidebar = readSidebar(readFileSync(path.join(docsRoot, 'sidebar.yml'), 'utf8'));
	const order = sidebarPages(sidebar.entries);
	const sections = sidebarSections(sidebar.entries);

	// First pass: parse every page, so links can be checked against every page's headings.
	const pages = new Map();

	for (const file of markdownFiles(docsRoot)) {
		const name = file.slice(0, -'.md'.length);
		const sourcePath = path.posix.join(docsDir, file);
		const parsed = attempt(() =>
			parsePage(readFileSync(path.join(docsRoot, file), 'utf8'), sourcePath),
		);

		if (parsed) {
			pages.set(name, { ...parsed, name, sourcePath });
		}

		if (!order.includes(name)) {
			problems.push(`${sourcePath}: not listed in ${docsDir}/sidebar.yml`);
		}

		// The site nests pages, so a page in a folder needs that folder's section page above it.
		const folder = path.posix.dirname(name);

		if (folder.includes('/')) {
			problems.push(`${sourcePath}: docs can be at most one folder deep`);
		} else if (folder !== '.' && !sections.some((section) => section.section === folder)) {
			problems.push(
				`${sourcePath}: ${docsDir}/sidebar.yml has no group with "section: ${folder}" for this folder`,
			);
		}
	}

	for (const name of order) {
		if (!existsSync(path.join(docsRoot, `${name}.md`))) {
			problems.push(
				`${docsDir}/sidebar.yml: lists "${name}", but ${docsDir}/${name}.md does not exist`,
			);
		}
	}

	for (const link of clientLinks) {
		const page = pages.get(link.path.replace(/^docs\//, ''));

		if (page && !page.headings.some((heading) => heading.id === link.anchor)) {
			problems.push(
				`${page.sourcePath}: installed Harper clients link to #${link.anchor}, and no heading has that id`,
			);
		}
	}

	// Second pass: render.
	const entries = [];

	for (const page of pages.values()) {
		const context = {
			file: page.sourcePath,

			resolveLink(url, node) {
				const fail = (message) => {
					throw new DocsProblem(message, page.sourcePath, node);
				};
				const checkAnchor = (target, fragment) => {
					if (fragment && !target.headings.some((heading) => heading.id === fragment)) {
						fail(`link to #${fragment}, and ${target.sourcePath} has no heading with that id`);
					}
				};

				if (/^[a-z][a-z0-9+.-]*:/i.test(url)) {
					return /^(https?|mailto):/i.test(url) ? url : fail(`link type not allowed: ${url}`);
				}
				if (url.startsWith('#')) {
					checkAnchor(page, decodeURIComponent(url.slice(1)));
					return url;
				}
				if (url.startsWith('//')) {
					return fail(`write the link with https: ${url}`);
				}
				if (url.startsWith('/')) {
					return url;
				}

				const [relative, fragment = ''] = url.split('#');
				const target = path.resolve(
					path.dirname(path.join(root, page.sourcePath)),
					decodeURI(relative),
				);
				const inRepo = path.relative(root, target);

				if (inRepo.startsWith('..')) {
					return fail(`link leaves the repository: ${url}`);
				}

				const inDocs = path.relative(docsRoot, target);

				if (!inDocs.startsWith('..') && inDocs.endsWith('.md')) {
					const linked = pages.get(inDocs.slice(0, -'.md'.length).split(path.sep).join('/'));

					if (!linked) {
						return existsSync(target) ? url : fail(`link to a page that does not exist: ${url}`);
					}

					checkAnchor(linked, decodeURIComponent(fragment));
					return `/docs/${linked.name}${fragment ? `#${fragment}` : ''}`;
				}

				if (!existsSync(target)) {
					return fail(`link to a file that does not exist: ${url}`);
				}

				// Anything else in the repository is shown on GitHub.
				const kind = statSync(target).isDirectory() ? 'tree' : 'blob';
				return `https://github.com/${repository}/${kind}/${branch}/${inRepo.split(path.sep).join('/')}${
					fragment ? `#${fragment}` : ''
				}`;
			},

			resolveImage(url, node) {
				if (/^https:\/\//i.test(url)) {
					return url;
				}
				if (url.startsWith('/') && existsSync(path.join(root, staticDir, url))) {
					return url;
				}

				throw new DocsProblem(
					`image ${url} not found. Docs images live in ${staticDir}/images and are written as /images/name.png`,
					page.sourcePath,
					node,
				);
			},

			include(file, node) {
				const fail = (message) => {
					throw new DocsProblem(message, page.sourcePath, node);
				};
				const target = path.resolve(root, file);

				if (!existsSync(target) || !statSync(target).isFile()) {
					return fail(`file=${file} does not exist. Paths start at the repository root.`);
				}

				const inRepo = path.relative(root, realpathSync(target));

				if (inRepo.startsWith('..') || inRepo.split(path.sep)[0] === '.git') {
					return fail(`file=${file} is outside the repository`);
				}
				if (statSync(target).size > MAX_INCLUDE_BYTES) {
					return fail(`file=${file} is too large to show on a page`);
				}

				return readFileSync(target, 'utf8');
			},

			rulesTable() {
				if (!rules) {
					throw new DocsProblem(
						'this page shows the rules, and they were not built',
						page.sourcePath,
					);
				}

				return rulesTable(rules);
			},
		};

		const content = attempt(() => renderPage(page.tree, context));

		if (content !== null) {
			entries.push({
				path: `docs/${page.name}`,
				title: page.title,
				description: page.description,
				source_path: page.sourcePath,
				content,
			});
		}
	}

	if (problems.length > 0) {
		throw new DocsBuildError(problems);
	}

	// The section pages: /docs and one per sidebar group that names a section.
	const descriptions = new Map(entries.map((entry) => [`/${entry.path}`, entry.description]));

	for (const section of sections) {
		descriptions.set(`/docs/${section.section}`, section.description ?? '');
		entries.push({
			path: `docs/${section.section}`,
			title: section.title,
			description: section.description ?? '',
			source_path: null,
			content: indexBlock(sectionLinks(section), descriptions),
		});
	}

	entries.push({
		path: 'docs',
		title: sidebar.title,
		description: sidebar.description,
		source_path: null,
		content: indexBlock(rootLinks(sidebar.entries), descriptions),
	});

	// Reading order, with each section page just before its first page.
	const position = new Map(order.map((name, index) => [`docs/${name}`, (index + 1) * 10]));

	for (const section of sections) {
		const first = section.items.find((link) => link.page);
		position.set(`docs/${section.section}`, first ? position.get(`docs/${first.page}`) - 5 : 0);
	}

	const pagesOut = entries
		.map((entry) => ({ ...entry, order: position.get(entry.path) ?? 0 }))
		.sort(
			(a, b) => depth(a.path) - depth(b.path) || a.order - b.order || a.path.localeCompare(b.path),
		)
		.map(withHash);

	const reference = referenceDir ? readReference(referenceDir).map(withHash) : [];
	const sidebarOut = themeSidebar(sidebar.entries);

	return {
		format: BUNDLE_FORMAT,
		source: { repository, branch, commit },
		sidebar: sidebarOut,
		sidebar_hash: hash(sidebarOut),
		pages: pagesOut,
		reference,
	};
}

function markdownFiles(directory, prefix = '') {
	return readdirSync(path.join(directory, prefix), { withFileTypes: true })
		.flatMap((entry) => {
			const relative = path.posix.join(prefix, entry.name);

			if (entry.isDirectory()) {
				return markdownFiles(directory, relative);
			}

			return entry.name.endsWith('.md') ? [relative] : [];
		})
		.sort();
}

const depth = (pagePath) => pagePath.split('/').length;

function hash(value) {
	return createHash('sha256').update(JSON.stringify(value)).digest('hex');
}

/** The hash covers every field, so a changed title or position is a change too. */
function withHash(entry) {
	return { ...entry, hash: hash(entry) };
}
