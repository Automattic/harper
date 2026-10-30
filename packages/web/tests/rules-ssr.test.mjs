import assert from 'node:assert/strict';
import test from 'node:test';

// Run against a fresh server to check the first response before client JavaScript runs.
const origin = process.env.HARPER_TEST_ORIGIN ?? 'http://127.0.0.1:3000';

async function assertServerRenderedRules() {
	const response = await fetch(new URL('/docs/rules', origin));
	assert.equal(response.status, 200);
	const html = await response.text();
	const tableBody = html.match(/<tbody\b[^>]*>([\s\S]*?)<\/tbody>/i)?.[1];
	assert.ok(tableBody, 'The rules table must be present in the HTTP response.');
	assert.match(tableBody, /SpellCheck/, 'Rule names must be present without client JavaScript.');
	assert.ok((tableBody.match(/<tr\b/g) ?? []).length > 0, 'The rules table must contain rows.');
	assert.match(tableBody, /✔️|❌/, 'Rule defaults must be present without client JavaScript.');
	assert.match(
		tableBody,
		/spelling/i,
		'Rule descriptions must be present without client JavaScript.',
	);
}

test('the first rules response includes the catalog without client JavaScript', async () => {
	await assertServerRenderedRules();
});

test('subsequent rules responses keep the server-rendered catalog', async () => {
	await assertServerRenderedRules();
});
