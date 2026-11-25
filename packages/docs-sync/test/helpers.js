import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { buildBundle } from '../src/build.js';

const SIDEBAR = `title: Documentation
description: Everything about the example.
sidebar:
  - items:
      - { title: About, page: about }
  - title: Guides
    section: guides
    description: Step by step.
    items:
      - { title: First, page: guides/first }
      - { title: Elsewhere, href: "https://example.com/" }
`;

export const page = (title, body) =>
	`---\ntitle: ${title}\ndescription: About ${title}.\n---\n\n# ${title}\n\n${body}\n`;

/** A throwaway repository with two pages, a sidebar and whatever else the test adds. */
export function fixture(files = {}) {
	const repo = mkdtempSync(path.join(tmpdir(), 'docs-sync-'));
	const all = {
		'docs/sidebar.yml': SIDEBAR,
		'docs/about.md': page('About', 'Hello.'),
		'docs/guides/first.md': page('First', '## Getting Started\n\nText.'),
		...files,
	};

	for (const [file, contents] of Object.entries(all)) {
		mkdirSync(path.dirname(path.join(repo, file)), { recursive: true });
		writeFileSync(path.join(repo, file), contents);
	}

	return repo;
}

export function build(files, options = {}) {
	return buildBundle({ repo: fixture(files), staticDir: 'static', ...options });
}

export function contentOf(bundle, pagePath) {
	return bundle.pages.find((entry) => entry.path === pagePath).content;
}

/** Build one page body and return its content. */
export function render(body, files = {}, options = {}) {
	return contentOf(
		build({ 'docs/about.md': page('About', body), ...files }, options),
		'docs/about',
	);
}
