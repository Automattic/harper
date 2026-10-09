import type { BrowserContext, Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { getBackground, openExtensionPage } from './testUtils';

const IMPORTED_WORD = 'zyxquorblat';
const SETTLED_MARKER_TITLE =
	'Set all rules in the Style and Redundancy category to their default, on, or off state.';

function settingsFile(overrides: Record<string, unknown> = {}) {
	return JSON.stringify({
		schema_version: 1,
		exported_at: '2026-10-09T10:00:00.000Z',
		source_app: 'harper-firefox',
		harper_version: '0.0.0',
		core: {
			dialect: 'British',
			lint_config: { RepeatedWords: false },
			user_dictionary: [IMPORTED_WORD],
			ignored_lints: '{"context_hashes":[18446744073709551615]}',
		},
		extension: {
			domain_status: { 'import.example': true },
			default_enabled: false,
			delay: 321,
			activation_key: 'shift',
			hotkey: { modifiers: ['Alt'], key: 'k' },
			isolate_english: false,
		},
		...overrides,
	});
}

/** The options page writes its loaded state back to storage, so let that settle first. */
async function openSettledOptionsPage(context: BrowserContext, page: Page) {
	await getBackground(context);
	await openExtensionPage(context, page, 'options.html');
	await expect(page.getByTitle(SETTLED_MARKER_TITLE)).toBeVisible({ timeout: 15000 });
	await page.waitForTimeout(1000);
}

async function send(page: Page, message: Record<string, unknown>) {
	return await page.evaluate((m) => chrome.runtime.sendMessage(m), message);
}

async function readStorage(page: Page, keys: string[]) {
	return await page.evaluate((k) => chrome.storage.local.get(k), keys);
}

test.describe('settings import and export', () => {
	test.describe.configure({ mode: 'serial' });
	test.setTimeout(90_000);
	test.skip(
		({ browserName }) => browserName === 'firefox',
		'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
	);

	test('imports a file, applies it to the linter, and exports it again', async ({
		context,
		page,
	}) => {
		await openSettledOptionsPage(context, page);

		const resp = await send(page, {
			kind: 'importSettings',
			json: settingsFile(),
			mode: 'merge',
			includeExtension: true,
		});
		expect(resp).toEqual({ kind: 'importSettings', ok: true });

		const stored = await readStorage(page, [
			'dialect',
			'delay',
			'activationKey',
			'userDictionary',
			'lintConfig',
			'ignoredLints',
			'domainStatus import.example',
		]);
		expect(stored.dialect).toBe(1);
		expect(stored.delay).toBe(321);
		expect(stored.activationKey).toBe('shift');
		expect(stored.userDictionary).toContain(IMPORTED_WORD);
		expect(JSON.parse(stored.lintConfig).RepeatedWords).toBe(false);
		expect(stored.ignoredLints).toContain('18446744073709551615');
		expect(stored['domainStatus import.example']).toBe(true);

		const lintResp = await send(page, {
			kind: 'lint',
			domain: 'import.example',
			text: `This is ${IMPORTED_WORD}.`,
		});
		const lints = Object.values(lintResp.lints as Record<string, unknown[]>).flat();
		expect(lints).toHaveLength(0);

		const exported = (await send(page, { kind: 'exportSettings' })).settings;
		expect(exported.core.dialect).toBe('British');
		expect(exported.core.user_dictionary).toContain(IMPORTED_WORD);
		expect(exported.core.lint_config.RepeatedWords).toBe(false);
		expect(exported.core.ignored_lints).toContain('18446744073709551615');
		expect(exported.extension.domain_status['import.example']).toBe(true);
	});

	test('rejects a bad file without changing anything', async ({ context, page }) => {
		await openSettledOptionsPage(context, page);

		const before = await readStorage(page, ['delay', 'dialect', 'userDictionary']);
		const resp = await send(page, {
			kind: 'importSettings',
			json: settingsFile({ schema_version: 99 }),
			mode: 'replace',
			includeExtension: true,
		});

		expect(resp.ok).toBe(false);
		expect(resp.error).toMatch(/schema_version/);
		expect(await readStorage(page, ['delay', 'dialect', 'userDictionary'])).toEqual(before);
	});
});
