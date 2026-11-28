import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { audienceFor, collect, githubToken, publish } from './publish-docs.mjs';

/** A throwaway repository holding the given files. */
function repository(files) {
	const root = mkdtempSync(path.join(tmpdir(), 'publish-docs-'));

	for (const [file, contents] of Object.entries(files)) {
		mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
		writeFileSync(path.join(root, file), contents);
	}

	return root;
}

const rules = async () => ({ descriptions: { SpellCheck: '<p>Spelling.</p>' }, defaultConfig: {} });

test('collect gathers the docs, the files they show, the rules and the API reference', async () => {
	const root = repository({
		'docs/about.md': '# About\n\n```ts file=src/linter.ts\n```\n',
		'docs/guides/first.md': '# First\n',
		'docs/sidebar.json': '{}',
		'docs/notes.txt': 'not published',
		'src/linter.ts': 'export {};\n',
		'src/unrelated.ts': 'not published',
		'ref/index.html': '<html></html>',
		'ref/notes.txt': 'not published',
	});

	const files = await collect(root, { rules, referenceDir: path.join(root, 'ref') });

	assert.deepEqual(Object.keys(files).sort(), [
		'docs/about.md',
		'docs/guides/first.md',
		'docs/sidebar.json',
		'generated/reference/index.html',
		'generated/rules.json',
		'src/linter.ts',
	]);
	assert.equal(files['src/linter.ts'], 'export {};\n');
	assert.equal(files['docs/about.md'], '# About\n\n```ts file=src/linter.ts\n```\n');
	assert.deepEqual(JSON.parse(files['generated/rules.json']), await rules());
});

test('a page cannot pull in a file from outside the repository or from .git', async () => {
	const root = repository({
		'docs/about.md':
			'```txt file=../outside.txt\n```\n\n```txt file=.git/config\n```\n\n```txt file=missing.txt\n```\n',
		'.git/config': 'token',
		'ref/index.html': '',
	});

	writeFileSync(path.join(root, '../outside.txt'), 'outside');

	const files = await collect(root, { rules, referenceDir: path.join(root, 'ref') });

	assert.deepEqual(Object.keys(files).sort(), [
		'docs/about.md',
		'generated/reference/index.html',
		'generated/rules.json',
	]);
});

const files = { 'docs/about.md': 'about', 'docs/sidebar.json': '{}', 'Cargo.toml': '[workspace]' };
const sha256 = (text) => createHash('sha256').update(text).digest('hex');

/** A site that answers each step from a script and records what it was sent. */
function fakeSite(answers) {
	const calls = [];

	return {
		calls,
		async fetch(url, init) {
			const step = new URL(url).searchParams.get('rest_route').split('/').pop();
			const answer = answers[step];

			calls.push({ step, url: String(url), headers: init.headers, body: JSON.parse(init.body) });

			return {
				ok: (answer.status ?? 200) < 300,
				status: answer.status ?? 200,
				text: async () => JSON.stringify(answer.body),
			};
		},
	};
}

const run = (site, options = {}) =>
	publish(files, {
		url: 'https://Docs.Example.com',
		commit: 'abc1234',
		token: async (audience) => `token-for-${audience}`,
		fetch: site.fetch,
		log: () => {},
		...options,
	});

test('the audience names the site, so a token for one site is useless on another', () => {
	assert.equal(
		audienceFor('https://WriteWithHarper.com/docs'),
		'harper-docs-sync:writewithharper.com',
	);
	assert.equal(audienceFor('http://localhost:8899'), 'harper-docs-sync:localhost');
});

test('publish sends every hash, then only the files the site lacks', async () => {
	const site = fakeSite({
		plan: { body: { needed: ['docs/about.md'], remove: [], unchanged: 2 } },
		files: { body: { stored: ['docs/about.md'], failed: {} } },
		finish: { body: { updated: ['docs/about'], trashed: [], removed: [], warnings: [] } },
	});

	await run(site);

	assert.deepEqual(
		site.calls.map((call) => call.step),
		['plan', 'files', 'finish'],
	);
	assert.deepEqual(site.calls[0].body, {
		commit: 'abc1234',
		files: {
			'docs/about.md': sha256('about'),
			'docs/sidebar.json': sha256('{}'),
			'Cargo.toml': sha256('[workspace]'),
		},
	});
	assert.deepEqual(site.calls[1].body.files, [{ path: 'docs/about.md', content: 'about' }]);
	assert.deepEqual(site.calls[2].body, site.calls[0].body);
	assert.equal(
		site.calls[0].headers.authorization,
		'Bearer token-for-harper-docs-sync:docs.example.com',
	);
	assert.equal(site.calls[0].url, 'https://docs.example.com/?rest_route=/harper-docs/v1/plan');
});

test('when nothing changed, nothing is uploaded', async () => {
	const site = fakeSite({
		plan: { body: { needed: [], remove: [], unchanged: 3 } },
		finish: { body: { updated: [], trashed: [], removed: [], warnings: [] } },
	});

	await run(site);

	assert.deepEqual(
		site.calls.map((call) => call.step),
		['plan', 'finish'],
	);
});

test('a dry run asks what would change and sends nothing else', async () => {
	const site = fakeSite({
		plan: { body: { needed: ['docs/about.md'], remove: [], unchanged: 2 } },
	});

	await run(site, { dryRun: true });

	assert.equal(site.calls.length, 1);
});

test('a refusal from the site fails the run with its message', async () => {
	const site = fakeSite({
		plan: { status: 401, body: { message: 'The sync token was not accepted: wrong ref.' } },
	});

	await assert.rejects(run(site), /plan: The sync token was not accepted: wrong ref\./);
});

test('what the site noticed is shown as warnings on the workflow run', async () => {
	const lines = [];
	const site = fakeSite({
		plan: { body: { needed: [], remove: [], unchanged: 3 } },
		finish: {
			body: { updated: [], trashed: [], removed: [], warnings: ['docs/about.md: a broken link'] },
		},
	});

	await run(site, { log: (line) => lines.push(line) });

	assert.ok(lines.includes('::warning::docs/about.md: a broken link'));
});

test('in GitHub Actions the token comes from the run, with the site as its audience', async () => {
	const requests = [];
	const token = await githubToken(
		'harper-docs-sync:example.com',
		{
			ACTIONS_ID_TOKEN_REQUEST_URL: 'https://actions.example/token?v=1',
			ACTIONS_ID_TOKEN_REQUEST_TOKEN: 'request-token',
		},
		async (url, init) => {
			requests.push({ url, init });
			return { ok: true, json: async () => ({ value: 'the-jwt' }) };
		},
	);

	assert.equal(token, 'the-jwt');
	assert.equal(
		requests[0].url,
		'https://actions.example/token?v=1&audience=harper-docs-sync%3Aexample.com',
	);
	assert.equal(requests[0].init.headers.authorization, 'bearer request-token');
	await assert.rejects(githubToken('x', {}), /id-token: write/);
});
