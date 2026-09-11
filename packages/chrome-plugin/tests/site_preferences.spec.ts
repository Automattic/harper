import { expect, test } from './fixtures';
import {
	assertPageHasNHighlights,
	getBackground,
	getStoredDomainStatus,
	getTextarea,
	openOptionsSection,
	replaceEditorContent,
	setStoredDomainStatus,
} from './testUtils';

test.skip(
	({ browserName }) => browserName === 'firefox',
	'Firefox MV3 background context is not exposed reliably in playwright-webextext.',
);

test('site preferences: manage bundled defaults, custom sites, and lint gating', async ({
	context,
	page,
}) => {
	const TEST_PAGE_URL = 'http://localhost:8081/simple_textarea.html';

	// Bundled defaults: bulk disable, then re-enable one.
	await openOptionsSection(context, page, 'Site Preferences');
	await expect(page.getByRole('checkbox', { name: 'docs.google.com' })).toBeVisible();
	await page.getByRole('button', { name: 'Disable All Default Sites' }).click();

	await expect
		.poll(async () => {
			const background = await getBackground(context);
			return await background.evaluate(async () => {
				const statuses = await chrome.storage.local.get(null);
				const domainStatuses = Object.entries(statuses).filter(([key]) =>
					key.startsWith('domainStatus '),
				);
				return (
					domainStatuses.length > 0 && domainStatuses.every(([, enabled]) => enabled === false)
				);
			});
		})
		.toBe(true);
	await expect(page.getByRole('button', { name: 'Enable All Default Sites' })).toBeVisible();

	// Defaults always come from the defaults list; an override never duplicates them.
	await expect(page.getByRole('checkbox', { name: 'www.youtube.com' })).toHaveCount(1);
	await expect(page.getByRole('checkbox', { name: 'www.youtube.com' })).not.toBeChecked();

	await page.getByRole('checkbox', { name: 'docs.google.com' }).check();
	await expect.poll(() => getStoredDomainStatus(context, 'docs.google.com')).toBe(true);

	// Non-default overrides show in the same list (enabled or not); reload to re-fetch.
	await setStoredDomainStatus(context, 'example.com', true);
	await setStoredDomainStatus(context, 'example.org', false);
	await openOptionsSection(context, page, 'Site Preferences');

	const customDomain = page.getByRole('checkbox', { name: 'example.com' });
	await expect(customDomain).toBeChecked();
	const disabledCustomDomain = page.getByRole('checkbox', { name: 'example.org', exact: true });
	await expect(disabledCustomDomain).toBeVisible();
	await expect(disabledCustomDomain).not.toBeChecked();

	// Non-default sites carry no "Default" badge, while bundled defaults do.
	await expect(
		page.getByRole('checkbox', { name: 'example.org', exact: true }).locator('..'),
	).not.toContainText('Default');
	await expect(page.getByRole('checkbox', { name: 'docs.google.com' }).locator('..')).toContainText(
		'Default',
	);

	await customDomain.uncheck();
	await expect.poll(() => getStoredDomainStatus(context, 'example.com')).toBe(false);

	// Removing a custom site deletes its override.
	await page.getByRole('button', { name: 'Remove example.com' }).click();
	await expect.poll(() => getStoredDomainStatus(context, 'example.com')).toBeUndefined();

	// Defaults have no per-site Remove; re-enable via bulk Reset All.
	const localhostCheckbox = page.getByRole('checkbox', { name: 'localhost' });
	await expect.poll(() => getStoredDomainStatus(context, 'localhost')).toBe(false);

	// Bulk Reset All to Defaults: re-enables defaults (stored true) and removes remaining customs.
	page.once('dialog', async (dialog) => await dialog.accept());
	await page.getByRole('button', { name: 'Reset All to Defaults' }).click();
	await expect.poll(() => getStoredDomainStatus(context, 'localhost')).toBe(true);
	await expect.poll(() => getStoredDomainStatus(context, 'example.org')).toBeUndefined();
	await expect(page.getByRole('button', { name: 'Disable All Default Sites' })).toBeVisible();
	// Re-disable for lint gating.
	await localhostCheckbox.uncheck();
	await expect.poll(() => getStoredDomainStatus(context, 'localhost')).toBe(false);

	await page.goto(TEST_PAGE_URL);
	await replaceEditorContent(getTextarea(page), 'This is an test');
	await page.waitForTimeout(4000);
	await assertPageHasNHighlights(page, 0);

	await openOptionsSection(context, page, 'Site Preferences');
	await localhostCheckbox.check();
	await expect.poll(() => getStoredDomainStatus(context, 'localhost')).toBe(true);

	await page.goto(TEST_PAGE_URL);
	await replaceEditorContent(getTextarea(page), 'This is an test');
	await assertPageHasNHighlights(page, 1);
});
