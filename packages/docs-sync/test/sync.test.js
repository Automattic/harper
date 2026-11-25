import assert from 'node:assert/strict';
import { test } from 'node:test';
import { audienceFor, githubToken, sync } from '../src/sync.js';

const bundle = {
	source: { repository: 'Automattic/harper', branch: 'master', commit: 'abc1234' },
	sidebar: [{ title: 'About', to: '/docs/about' }],
	sidebar_hash: 's'.repeat(64),
	pages: [
		{ path: 'docs', hash: '1'.repeat(64), content: 'root' },
		{ path: 'docs/about', hash: '2'.repeat(64), content: 'about' },
	],
	reference: [{ file: 'index.html', hash: '3'.repeat(64), content: 'index' }],
};

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

const plan = (overrides = {}) => ({
	pages: { needed: ['docs/about'], remove: [], unchanged: 1 },
	reference: { needed: [], remove: [], unchanged: 1 },
	sidebar_changed: false,
	...overrides,
});

const run = (site, options = {}) =>
	sync(bundle, {
		url: 'https://Docs.Example.com',
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

test('only the entries the site lacks are uploaded', async () => {
	const site = fakeSite({
		plan: { body: plan() },
		items: { body: { stored: ['docs/about'], filtered: [], failed: {} } },
		finish: { body: { trashed: [], deleted: [], sidebar_changed: false } },
	});

	const result = await run(site);

	assert.deepEqual(
		site.calls.map((call) => call.step),
		['plan', 'items', 'finish'],
	);
	assert.deepEqual(site.calls[0].body.pages, [
		{ path: 'docs', hash: '1'.repeat(64) },
		{ path: 'docs/about', hash: '2'.repeat(64) },
	]);
	assert.deepEqual(site.calls[1].body.pages, [bundle.pages[1]]);
	assert.deepEqual(site.calls[2].body.sidebar, bundle.sidebar);
	assert.equal(
		site.calls[0].headers.authorization,
		'Bearer token-for-harper-docs-sync:docs.example.com',
	);
	assert.equal(site.calls[0].url, 'https://docs.example.com/?rest_route=/harper-docs/v1/plan');
	assert.equal(result.sent, 1);
});

test('a dry run asks for the plan and sends nothing else', async () => {
	const site = fakeSite({ plan: { body: plan() } });

	await run(site, { dryRun: true });

	assert.deepEqual(
		site.calls.map((call) => call.step),
		['plan'],
	);
});

test('a refusal from the site fails the run with its message', async () => {
	const site = fakeSite({
		plan: {
			status: 401,
			body: {
				code: 'harper_docs_unauthorized',
				message: 'The sync token was not accepted: wrong ref.',
			},
		},
	});

	await assert.rejects(run(site), /plan: The sync token was not accepted: wrong ref\./);
});

test('markup the site removed is reported as a warning', async () => {
	const lines = [];
	const site = fakeSite({
		plan: { body: plan() },
		items: { body: { stored: ['docs/about'], filtered: ['docs/about'], failed: {} } },
		finish: { body: { trashed: [], deleted: [], sidebar_changed: false } },
	});

	await run(site, { log: (line) => lines.push(line) });

	assert.ok(
		lines.includes('::warning::The site removed markup it does not allow from docs/about.'),
	);
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
