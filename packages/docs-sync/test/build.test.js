import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { DocsBuildError } from '../src/build.js';
import { parseReferenceFile } from '../src/reference.js';
import { build, contentOf, page } from './helpers.js';

test('the bundle holds each page, the section pages and the sidebar', () => {
	const bundle = build();

	assert.deepEqual(
		bundle.pages.map((entry) => [entry.path, entry.title, entry.source_path]),
		[
			['docs', 'Documentation', null],
			['docs/about', 'About', 'docs/about.md'],
			['docs/guides', 'Guides', null],
			['docs/guides/first', 'First', 'docs/guides/first.md'],
		],
	);
	assert.equal(bundle.pages[1].description, 'About About.');
	assert.deepEqual(bundle.sidebar, [
		{ items: [{ title: 'About', to: '/docs/about' }] },
		{
			title: 'Guides',
			items: [
				{ title: 'First', to: '/docs/guides/first' },
				{ title: 'Elsewhere', to: 'https://example.com/' },
			],
		},
	]);
});

test('parents come before their children, and pages follow the sidebar order', () => {
	const bundle = build();
	const order = Object.fromEntries(bundle.pages.map((entry) => [entry.path, entry.order]));

	assert.ok(order['docs/about'] < order['docs/guides']);
	assert.ok(order['docs/guides'] < order['docs/guides/first']);
});

test('section pages list their children with descriptions', () => {
	const bundle = build();

	assert.match(
		contentOf(bundle, 'docs/guides'),
		/<li><a href="\/docs\/guides\/first"><strong>First<\/strong><\/a><br \/>About First.<\/li>/,
	);
	assert.match(
		contentOf(bundle, 'docs/guides'),
		/<li><a href="https:\/\/example.com\/"><strong>Elsewhere<\/strong><\/a><\/li>/,
	);
	assert.match(
		contentOf(bundle, 'docs'),
		/<li><a href="\/docs\/guides"><strong>Guides<\/strong><\/a><br \/>Step by step.<\/li>/,
	);
});

test('a hash changes when anything about the page changes', () => {
	const hashes = (files) =>
		Object.fromEntries(build(files).pages.map((entry) => [entry.path, entry.hash]));
	const before = hashes();
	const after = hashes({ 'docs/about.md': page('About', 'Hello, world.') });

	assert.notEqual(before['docs/about'], after['docs/about']);
	assert.equal(before['docs/guides/first'], after['docs/guides/first']);
	assert.deepEqual(before, hashes());
});

test('pages and the sidebar must agree', () => {
	assert.throws(
		() => build({ 'docs/orphan.md': page('Orphan', 'x') }),
		(error) =>
			error instanceof DocsBuildError &&
			/orphan\.md: not listed in docs\/sidebar\.yml/.test(error.message),
	);
});

test('links that installed clients open must keep working', () => {
	const clientLinks = [{ path: 'docs/guides/first', anchor: 'Getting-Started' }];

	assert.doesNotThrow(() => build({}, { clientLinks }));
	assert.throws(
		() => build({ 'docs/guides/first.md': page('First', '## Renamed') }, { clientLinks }),
		/installed Harper clients link to #Getting-Started/,
	);
});

test('every problem is reported in one run', () => {
	assert.throws(
		() =>
			build({
				'docs/about.md': page('About', '[x](./missing.md)'),
				'docs/guides/first.md': page('First', '<br>'),
			}),
		(error) => error.problems.length === 2,
	);
});

test('reference files keep their body and link to each other absolutely', () => {
	const entry = parseReferenceFile(
		'harper.js.linter.html',
		'<!doctype html>\n<html><head><title>linter - Harper</title></head>\n<body class="container">\n<p><a href="./index.html" >Home</a></p>\n</body>\n</html>',
	);

	assert.deepEqual(entry, {
		file: 'harper.js.linter.html',
		title: 'linter - Harper',
		content: '<p><a href="/docs/harperjs/ref/index.html">Home</a></p>',
	});
});

test('the reference is part of the bundle when it has been built', () => {
	const referenceDir = mkdtempSync(path.join(tmpdir(), 'docs-sync-ref-'));
	writeFileSync(
		path.join(referenceDir, 'index.html'),
		'<html><head><title>index - Harper</title></head><body><h2>API</h2></body></html>',
	);

	const bundle = build({}, { referenceDir });

	assert.equal(bundle.reference.length, 1);
	assert.equal(bundle.reference[0].file, 'index.html');
	assert.match(bundle.reference[0].hash, /^[0-9a-f]{64}$/);
});

test('a folder of pages needs a section in the sidebar', () => {
	const sidebar = `title: Documentation
sidebar:
  - items:
      - { title: About, page: about }
      - { title: First, page: guides/first }
`;

	assert.throws(
		() => build({ 'docs/sidebar.yml': sidebar }),
		/guides\/first\.md: docs\/sidebar\.yml has no group with "section: guides"/,
	);
});
