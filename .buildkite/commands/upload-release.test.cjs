const assert = require('node:assert/strict');
const { once } = require('node:events');
const { mkdtemp, rm, writeFile } = require('node:fs/promises');
const { createServer } = require('node:http');
const { tmpdir } = require('node:os');
const { basename, join } = require('node:path');
const { test } = require('node:test');
const { uploadRelease } = require('./upload-release.cjs');

async function fixture(t) {
	const directory = await mkdtemp(join(tmpdir(), 'Harper release '));
	t.after(() => rm(directory, { recursive: true, force: true }));
	const file = join(directory, 'Harper_9.0.0_x64-setup.exe');
	await writeFile(file, Buffer.from([0, 255, 1, 2]));
	const draft = {
		id: 123,
		tag_name: 'v9.0.0',
		draft: true,
		name: 'Release title',
		body: 'Maintainer release notes',
		html_url: 'https://github.com/example/harper/releases/123',
		assets: [],
	};
	const state = {
		releases: [draft],
		assets: [{ id: 1, name: 'Harper.dmg' }],
		calls: [],
		failure: undefined,
	};
	let nextId = 2;
	const request = async (url, options) => {
		const { method, body } = options;
		const { pathname, searchParams } = new URL(url);
		state.calls.push({ url, method });
		if (state.failure === method)
			return Response.json({ message: 'test failure' }, { status: 403 });
		if (method === 'GET') {
			if (pathname === '/repos/example/harper/releases/123') return Response.json(draft);
			const entries = pathname.endsWith('/assets') ? state.assets : state.releases;
			const offset = (Number(searchParams.get('page')) - 1) * 100;
			return Response.json(entries.slice(offset, offset + 100));
		}
		if (method === 'DELETE') {
			const id = Number(pathname.split('/').pop());
			state.assets = state.assets.filter((asset) => asset.id !== id);
			return new Response(null, { status: 204 });
		}
		assert.equal(method, 'POST');
		assert.equal(pathname, '/repos/example/harper/releases/123/assets');
		const asset = { id: nextId++, name: searchParams.get('name'), bytes: [...body] };
		state.assets.push(asset);
		return Response.json(asset, { status: 201 });
	};
	const options = {
		repository: 'example/harper',
		tag: 'v9.0.0',
		assets: [file],
		token: 'test-token',
		request,
		wait: t.mock.fn(async () => {}),
	};
	return { directory, file, draft, state, options };
}

test('uploads binary data and replaces only matching assets on a rerun over HTTP', async (t) => {
	const { file, draft, state, options } = await fixture(t);
	const metadata = structuredClone(draft);
	const server = createServer(async (request, response) => {
		try {
			assert.equal(request.headers.authorization, 'Bearer test-token');
			assert.equal(request.headers['x-github-api-version'], '2022-11-28');
			const chunks = [];
			for await (const chunk of request) chunks.push(chunk);
			if (request.method === 'POST') {
				assert.equal(request.headers['content-type'], 'application/octet-stream');
			}
			const result = await options.request(`https://api.github.com${request.url}`, {
				method: request.method,
				body: Buffer.concat(chunks),
			});
			response.writeHead(result.status, { 'Content-Type': 'application/json' });
			response.end(await result.text());
		} catch (error) {
			response.writeHead(500);
			response.end(error.message);
		}
	});
	server.listen(0, '127.0.0.1');
	await once(server, 'listening');
	t.after(() => new Promise((resolve) => server.close(resolve)));
	const request = (url, init) => {
		const parsed = new URL(url);
		return fetch(
			`http://127.0.0.1:${server.address().port}${parsed.pathname}${parsed.search}`,
			init,
		);
	};
	assert.equal(await uploadRelease({ ...options, request }), draft.html_url);
	await writeFile(file, Buffer.from([255, 0, 42]));
	await uploadRelease({ ...options, request });
	assert.deepEqual(state.releases, [metadata]);
	assert.deepEqual(state.assets, [
		{ id: 1, name: 'Harper.dmg' },
		{ id: 3, name: basename(file), bytes: [255, 0, 42] },
	]);
	assert.equal(state.calls.filter((call) => call.method === 'DELETE').length, 1);
});

test('attaches to the draft holding the macOS dmg when Actions created another draft', async (t) => {
	const { draft, state, options } = await fixture(t);
	state.releases = [
		{ ...draft, id: 124, assets: [{ id: 8, name: 'harper-chrome-plugin.zip' }] },
		{ ...draft, assets: [{ id: 9, name: 'Harper_9.0.0_universal.dmg' }] },
	];
	assert.equal(await uploadRelease(options), draft.html_url);
	const uploads = state.calls.filter((call) => call.method === 'POST');
	assert.equal(uploads.length, 1);
	assert.match(uploads[0].url, /\/releases\/123\/assets\?/);
});

for (const scenario of [
	'missing',
	'other-tag',
	'duplicate',
	'duplicate-dmg',
	'published',
	'published-and-draft',
]) {
	test(`rejects ${scenario} releases before any writes`, async (t) => {
		const { draft, state, options } = await fixture(t);
		const dmg = { id: 9, name: 'Harper_9.0.0_universal.dmg' };
		state.releases = {
			missing: [],
			'other-tag': [{ ...draft, tag_name: 'v8.0.0' }],
			duplicate: [draft, { ...draft, id: 124 }],
			'duplicate-dmg': [
				{ ...draft, assets: [dmg] },
				{ ...draft, id: 124, assets: [dmg] },
			],
			published: [{ ...draft, draft: false }],
			'published-and-draft': [draft, { ...draft, id: 124, draft: false }],
		}[scenario];
		await assert.rejects(
			uploadRelease(options),
			/No draft release|Expected one draft|published release/,
		);
		assert.ok(state.calls.every((call) => call.method === 'GET'));
	});
}

test('paginates releases and assets before selecting and replacing', async (t) => {
	const { file, draft, state, options } = await fixture(t);
	state.releases = [
		...Array.from({ length: 100 }, (_, id) => ({ id, tag_name: `old-${id}` })),
		draft,
	];
	state.assets = [
		...Array.from({ length: 100 }, (_, id) => ({ id, name: `other-${id}` })),
		{ id: 200, name: basename(file) },
	];
	await uploadRelease(options);
	assert.equal(state.assets.length, 101);
	assert.equal(state.assets.filter((asset) => asset.name === basename(file)).length, 1);
	assert.ok(
		state.calls.some(
			(call) => call.url.endsWith('/releases/assets/200') && call.method === 'DELETE',
		),
	);
});

test('does not overlook a conflicting release on a later page', async (t) => {
	const { draft, state, options } = await fixture(t);
	state.releases = [
		draft,
		...Array.from({ length: 99 }, (_, id) => ({ id, tag_name: `old-${id}` })),
		{ ...draft, id: 124 },
	];
	await assert.rejects(uploadRelease(options), /found 2/);
	assert.ok(state.calls.every((call) => call.method === 'GET'));
});

for (const scenario of [
	'no-token',
	'no-tag',
	'no-assets',
	'missing-file',
	'directory',
	'duplicate-name',
]) {
	test(`rejects ${scenario} before contacting GitHub`, async (t) => {
		const { directory, file, state, options } = await fixture(t);
		const overrides = {
			'no-token': { token: ' ' },
			'no-tag': { tag: '' },
			'no-assets': { assets: [] },
			'missing-file': { assets: [file, join(directory, 'missing.exe')] },
			directory: { assets: [directory] },
			'duplicate-name': { assets: [file, file] },
		};
		await assert.rejects(uploadRelease({ ...options, ...overrides[scenario] }));
		assert.deepEqual(state.calls, []);
	});
}

for (const method of ['GET', 'DELETE', 'POST']) {
	test(`propagates ${method} failures without continuing uploads`, async (t) => {
		const { file, state, options } = await fixture(t);
		state.assets.push({ id: 2, name: basename(file) });
		state.failure = method;
		await assert.rejects(uploadRelease(options), /HTTP 403: test failure/);
		assert.equal(state.calls.at(-1).method, method);
		assert.equal(options.wait.mock.callCount(), 0);
	});
}

for (const failure of ['upload', 'delete', 'list']) {
	test(`recovers from a transient ${failure} failure by re-listing assets`, async (t) => {
		const { file, state, options } = await fixture(t);
		state.assets.push({ id: 200, name: basename(file) });
		let failed = false;
		const request = async (url, init) => {
			const result = await options.request(url, init);
			const matches = {
				upload: init.method === 'POST',
				delete: init.method === 'DELETE',
				list: init.method === 'GET' && url.includes('/assets?'),
			};
			if (!failed && matches[failure]) {
				failed = true;
				if (failure === 'upload') {
					state.assets.at(-1).state = 'starter';
					state.assets.at(-1).bytes = [];
				}
				return Response.json({ message: 'Temporary upstream failure' }, { status: 502 });
			}
			return result;
		};
		await uploadRelease({ ...options, request });
		assert.equal(failed, true);
		assert.equal(state.assets.length, 2);
		assert.deepEqual(state.assets[0], { id: 1, name: 'Harper.dmg' });
		assert.deepEqual(state.assets[1].bytes, [0, 255, 1, 2]);
		assert.equal(state.assets[1].state, undefined);
		assert.deepEqual(
			options.wait.mock.calls.map((call) => call.arguments),
			[[1000]],
		);
		assert.equal(
			state.calls.filter((call) => call.method === 'GET' && call.url.includes('/assets?')).length,
			2,
		);
	});
}

test('stops after three attempts when the upload keeps returning 5xx', async (t) => {
	const { state, options } = await fixture(t);
	const request = async (url, init) => {
		const result = await options.request(url, init);
		return init.method === 'POST'
			? Response.json({ message: 'Unavailable' }, { status: 503 })
			: result;
	};
	await assert.rejects(uploadRelease({ ...options, request }), /HTTP 503: Unavailable/);
	assert.equal(state.calls.filter((call) => call.method === 'POST').length, 3);
	assert.deepEqual(
		options.wait.mock.calls.map((call) => call.arguments),
		[[1000], [2000]],
	);
});

test('refuses to retry an upload if the draft has been published', async (t) => {
	const { draft, state, options } = await fixture(t);
	const request = async (url, init) => {
		const result = await options.request(url, init);
		if (init.method === 'POST') {
			draft.draft = false;
			return Response.json({ message: 'Unavailable' }, { status: 503 });
		}
		return result;
	};
	await assert.rejects(uploadRelease({ ...options, request }), /published release/);
	assert.equal(state.calls.filter((call) => call.method === 'POST').length, 1);
	assert.equal(state.calls.filter((call) => call.method === 'DELETE').length, 0);
});

test('includes GitHub validation details and does not retry 422 responses', async (t) => {
	const { options } = await fixture(t);
	const request = t.mock.fn(async () =>
		Response.json(
			{
				message: 'Validation Failed',
				errors: [{ resource: 'ReleaseAsset', field: 'name', code: 'already_exists' }],
			},
			{ status: 422 },
		),
	);
	await assert.rejects(
		uploadRelease({ ...options, request }),
		/HTTP 422: Validation Failed.*already_exists/,
	);
	assert.equal(request.mock.callCount(), 1);
	assert.equal(options.wait.mock.callCount(), 0);
});

test('reports the request and underlying network error', async (t) => {
	const { options } = await fixture(t);
	const request = async () => {
		throw new Error('fetch failed', { cause: new Error('connect ETIMEDOUT') });
	};
	await assert.rejects(
		uploadRelease({ ...options, request }),
		/GitHub GET .*fetch failed: connect ETIMEDOUT/,
	);
});
