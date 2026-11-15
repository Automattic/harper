import { expect, test } from './fixtures';

const origin = 'http://localhost:8081';
const otherOrigin = 'http://127.0.0.1:8081';
const seed = 'This is an test';
const style =
	'<style>body{margin:16px;font:18px Arial}iframe{display:block;width:800px;height:400px}#editor{padding:12px;border:1px solid;min-height:140px}input,div{margin:8px}</style>';

for (const scenario of ['parent', 'prevent-default', 'nested', 'alternating-origin'] as const) {
	test(`Closes an iframe popup from ${scenario} outside content`, async ({ context, page }) => {
		const ownerUrl = `${origin}/harper-iframe-popup-owner`;
		const middleOrigin = scenario === 'alternating-origin' ? otherOrigin : origin;
		const middleUrl = `${middleOrigin}/harper-iframe-popup-middle`;
		const topUrl = `${origin}/harper-iframe-popup-top`;
		const nested = scenario === 'nested' || scenario === 'alternating-origin';
		const preserveFocus = scenario === 'prevent-default' || scenario === 'alternating-origin';

		await context.route(ownerUrl, (route) =>
			route.fulfill({
				contentType: 'text/html; charset=utf-8',
				body: `<!doctype html><meta charset="utf-8">${style}<div id="editor" contenteditable="true" spellcheck="true">${seed}</div>`,
			}),
		);
		await context.route(middleUrl, (route) =>
			route.fulfill({
				contentType: 'text/html; charset=utf-8',
				body: `<!doctype html>${style}<iframe src="${ownerUrl}"></iframe>`,
			}),
		);
		await context.route(topUrl, (route) =>
			route.fulfill({
				contentType: 'text/html; charset=utf-8',
				body: `<!doctype html>${style}<input id="outside-input"><div id="outside" ${preserveFocus ? 'onpointerdown="event.preventDefault()"' : ''}>Outside the editor</div><iframe src="${nested ? middleUrl : ownerUrl}"></iframe>`,
			}),
		);

		await page.goto(topUrl);
		await expect.poll(() => page.frames().some((frame) => frame.url() === ownerUrl)).toBe(true);
		const frame = page.frames().find((candidate) => candidate.url() === ownerUrl)!;
		const editor = frame.locator('#editor');
		const highlight = frame.locator('#harper-highlight[data-problem-text="an"]');
		await expect(highlight).toHaveCount(1);
		await expect(highlight).toBeVisible();
		const originalHTML = await editor.innerHTML();
		const suggestion = frame.getByTitle('Click to replace "an" with "a"');

		const open = async () => {
			const box = await highlight.boundingBox();
			expect(box).not.toBeNull();
			await page.mouse.click(box!.x + box!.width / 2, box!.y + Math.min(8, box!.height / 2));
			await expect(suggestion).toBeVisible();
		};

		await open();
		await page.locator('#outside').click();
		await expect(suggestion).toBeHidden();
		await expect(editor).toHaveText(seed);
		expect(await editor.innerHTML()).toBe(originalHTML);

		// Outside dismissal must not disable genuine suggestion actions.
		await open();
		await suggestion.click();
		await expect(editor).toHaveText('This is a test');
		expect(await editor.innerHTML()).toBe(originalHTML.replace('an', 'a'));
	});
}
