import { fileURLToPath, URL as NodeURL } from 'node:url';
import { test as browserTest, type Page } from '@playwright/test';
import { SuggestionKind } from 'harper.js';
import { build } from 'vite';
import type { IgnorableLintBox } from '../../lint-framework/src/lint/Box';
import type PopupHandler from '../../lint-framework/src/lint/PopupHandler';
import SuggestionBox from '../../lint-framework/src/lint/SuggestionBox';
import { expect, test } from './fixtures';
import {
	clickHarperHighlight,
	getBackground,
	getHarperHighlights,
	getHarperSuggestionRows as rows,
} from './testUtils';

const URL = 'http://localhost:8081/suggestion_popup.html';
const TEXT = 'This is an test. I could of gone.';
const flyout = (page: Page) => page.locator('.harper-flyout');

/** Allow the extension's cold WASM startup, especially with concurrent Firefox profiles. */
async function waitForLints(page: Page, count: number) {
	await expect(getHarperHighlights(page)).toHaveCount(count, { timeout: 12000 });
}

/** Open a real extension popup after its lints have reached the expected count. */
async function openMenu(page: Page, text = TEXT, count = 2) {
	await page.goto(URL);
	await page.locator('#first').fill(text);
	await waitForLints(page, count);
	expect(await clickHarperHighlight(page)).toBe(true);
	await expect(rows(page)).not.toHaveCount(0);
	await expect(rows(page).first()).toBeFocused();
}

/** Assert the complete panel, rather than just its anchor, fits inside the viewport. */
async function expectPanelsInViewport(page: Page) {
	for (const selector of ['.harper-container', '.harper-flyout']) {
		await expect
			.poll(async () => {
				return page.locator(selector).evaluate((panel) => {
					const rect = panel.getBoundingClientRect();
					return (
						rect.left >= 0 &&
						rect.top >= 0 &&
						rect.right <= innerWidth &&
						rect.bottom <= innerHeight
					);
				});
			})
			.toBe(true);
	}
}

/** Wait for the popup's layout hooks before measuring its stationary flyout. */
async function flyoutPosition(page: Page) {
	return flyout(page).evaluate(async (panel) => {
		await new Promise<void>((resolve) => {
			requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
		});
		const { x, y } = panel.getBoundingClientRect();
		return { x, y };
	});
}

test('Shows only the clicked lint, excluding other lints in the same and other fields', async ({
	page,
}) => {
	await page.goto(URL);
	await page.locator('#first').fill(TEXT);
	await page.locator('#second').fill('She is an teacher.');
	await waitForLints(page, 3);
	const highlights = await getHarperHighlights(page).all();
	const bounds = await Promise.all(highlights.map((highlight) => highlight.boundingBox()));
	const firstField = (await page.locator('#first').boundingBox())!;
	const secondLint = bounds
		.filter((box) => box && box.y < firstField.y + firstField.height)
		.sort((a, b) => a!.x - b!.x)[1]!;
	await page.mouse.click(secondLint.x + secondLint.width / 2, secondLint.y + secondLint.height / 2);
	await expect(rows(page)).toHaveText(['could have›']);
	await expect(rows(page).first()).toBeFocused();
	await expect(flyout(page).getByTitle('Disable the ModalOf rule')).toBeVisible();
});

test('Hover selection sticks while crossing the gap, without moving either panel', async ({
	page,
}) => {
	await page.emulateMedia({ reducedMotion: 'reduce' });
	await openMenu(page, 'This is a tset.', 1);
	await expect(rows(page)).toHaveText(['test›', 'that›', 'the›']);
	await expect(rows(page).nth(1)).toHaveAttribute('title', 'Click to replace "tset" with "that"');
	const positionBefore = await flyoutPosition(page);
	const messageBefore = await flyout(page).locator('.harper-body').innerHTML();
	const listBefore = await page.locator('.harper-container').boundingBox();
	const hintBefore = await page.locator('.harper-hint-drawer').allTextContents();
	await rows(page).nth(1).hover();
	await expect(rows(page).nth(1)).toHaveAttribute('aria-expanded', 'true');
	await expect(flyout(page).getByTitle('Ignore this lint')).toBeVisible();
	expect(await flyoutPosition(page)).toEqual(positionBefore);
	expect(await flyout(page).locator('.harper-body').innerHTML()).toBe(messageBefore);
	const list = (await page.locator('.harper-container').boundingBox())!;
	await page.mouse.move(list.x + list.width + 3, list.y + 45);
	await expect(flyout(page)).toBeVisible();
	await expect(rows(page).nth(1)).toHaveAttribute('aria-expanded', 'true');
	expect(await page.locator('.harper-hint-drawer').allTextContents()).toEqual(hintBefore);
	await expect
		.poll(async () => (await page.locator('.harper-container').boundingBox())?.x)
		.toBe(listBefore!.x);
	await rows(page).nth(1).click();
	await expect(page.locator('#first')).toHaveValue('This is a that.');
	await expect(rows(page)).toHaveCount(0);
});

test('Keyboard navigation wraps, enters and leaves the flyout, and applies the selected row', async ({
	page,
}) => {
	await page.emulateMedia({ reducedMotion: 'reduce' });
	await openMenu(page, 'This is a tset.', 1);
	const positionBefore = await flyoutPosition(page);
	await page.keyboard.press('ArrowUp');
	await expect(rows(page).last()).toBeFocused();
	expect(await flyoutPosition(page)).toEqual(positionBefore);
	await page.keyboard.press('ArrowDown');
	await expect(rows(page).first()).toBeFocused();
	expect(await flyoutPosition(page)).toEqual(positionBefore);
	await page.keyboard.press('ArrowRight');
	await expect(flyout(page).getByTitle('Ignore this lint')).toBeFocused();
	await page.keyboard.press('ArrowLeft');
	await expect(rows(page).first()).toBeFocused();
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('Enter');
	await expect(page.locator('#first')).toHaveValue('This is a that.');
	await expect(rows(page)).toHaveCount(0);
});

test('Clicking a row applies only that lint', async ({ page }) => {
	await openMenu(page);
	await rows(page).first().click();
	await expect(page.locator('#first')).toHaveValue('This is a test. I could of gone.');
	await expect(rows(page)).toHaveCount(0);
});

test('Escape restores the textarea caret after hovering another row', async ({ page }) => {
	await openMenu(page, 'This is a tset.', 1);
	const selection = await page
		.locator('#first')
		.evaluate((field: HTMLTextAreaElement) => [field.selectionStart, field.selectionEnd]);
	await rows(page).last().hover();
	await page.keyboard.press('Escape');
	await expect(page.locator('#first')).toBeFocused();
	await expect(rows(page)).toHaveCount(0);
	await expect
		.poll(() =>
			page
				.locator('#first')
				.evaluate((field: HTMLTextAreaElement) => [field.selectionStart, field.selectionEnd]),
		)
		.toEqual(selection);
	// Closing ends the autofocus hook's DOM lifetime, even though RenderBox is reused.
	expect(await clickHarperHighlight(page)).toBe(true);
	await expect(rows(page).first()).toBeFocused();
	await page.getByRole('button', { name: 'Close', exact: true }).click();
	await expect(page.locator('#first')).toBeFocused();
});

test('Pointer-down outside either panel closes without stealing the clicked control focus', async ({
	page,
}) => {
	await openMenu(page);
	await page.locator('#outside').click();
	await expect(rows(page)).toHaveCount(0);
	await expect(page.locator('#outside')).toBeFocused();
});

test('Dismiss ignores only the selected lint, leaving other lints and the text untouched', async ({
	page,
}) => {
	await page.goto(URL);
	await page.locator('#first').fill(TEXT);
	await page.locator('#second').fill('She is an teacher.');
	await waitForLints(page, 3);
	expect(await clickHarperHighlight(page)).toBe(true);
	await expect(rows(page)).toHaveCount(1);
	await expect(page.getByRole('button', { name: 'Dismiss all', exact: true })).toHaveCount(0);
	await flyout(page).getByTitle('Ignore this lint').click();
	await waitForLints(page, 2);
	await expect(page.locator('#first')).toHaveValue(TEXT);
	await expect(page.locator('#second')).toHaveValue('She is an teacher.');
	await expect(page.locator('#first')).toBeFocused();
});

test('Spelling alternatives remain in the list and dictionary action remains in the flyout', async ({
	page,
}) => {
	await openMenu(page, 'This is a tset.', 1);
	await expect(rows(page)).toHaveText(['test›', 'that›', 'the›']);
	await expect(flyout(page).locator('.harper-replace')).toHaveCount(0);
	await expect(
		flyout(page).getByRole('button', { name: /^(Replace|Remove|Insert After)$/ }),
	).toHaveCount(0);
	await flyout(page).getByTitle('Add word to user dictionary').click();
	await waitForLints(page, 0);
	await expect(rows(page)).toHaveCount(0);
	await expect(page.locator('#first')).toHaveValue('This is a tset.');
});

test('A wrapped lint has one row even when it has several highlight rectangles', async ({
	page,
}) => {
	await page.goto(URL);
	const rich = page.locator('#rich');
	// Soft-wrap inside a single ModalOf span without changing its underlying text.
	await rich.evaluate((element) => {
		element.style.width = '90px';
	});
	await rich.fill('I could of gone.');
	await expect.poll(() => getHarperHighlights(page).count(), { timeout: 12000 }).toBeGreaterThan(1);
	const fragment = (await getHarperHighlights(page).last().boundingBox())!;
	await page.mouse.click(fragment.x + fragment.width / 2, fragment.y + fragment.height / 2);
	await expect(rows(page)).toHaveCount(1);
	await expect(rows(page).first()).toHaveText('could have›');
	await expect(rows(page).first()).toBeFocused();
	const initialList = (await page.locator('.harper-container').boundingBox())!;
	// Highlight DOM includes a border outside the underlying lint rectangle.
	const anchorGap = Math.min(
		Math.abs(initialList.y - fragment.y - fragment.height),
		Math.abs(fragment.y - initialList.y - initialList.height),
	);
	expect(anchorGap).toBeLessThanOrEqual(6);
	// Recomputing identical geometry must not move the anchor to the first wrapped line.
	await page.evaluate(
		() =>
			new Promise<void>((resolve) => {
				window.dispatchEvent(new Event('resize'));
				requestAnimationFrame(() =>
					requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
				);
			}),
	);
	await expect
		.poll(async () => (await page.locator('.harper-container').boundingBox())?.y)
		.toBeCloseTo(initialList.y, 0);
});

test('Unrelated lints do not add replacement rows to the popup', async ({ page }) => {
	await page.goto(URL);
	await page.locator('#first').evaluate((element) => {
		element.style.height = '620px';
	});
	await page
		.locator('#first')
		.fill(Array.from({ length: 20 }, () => 'This is an test.').join('\n'));
	await waitForLints(page, 20);
	expect(await clickHarperHighlight(page)).toBe(true);
	await expect(rows(page)).toHaveText(['a›']);
	await expect(rows(page).first()).toBeFocused();
	await expectPanelsInViewport(page);
});

test('Flips near the right and bottom viewport edges', async ({ page }) => {
	await page.emulateMedia({ reducedMotion: 'reduce' });
	await page.goto(URL);
	await page.locator('#first').evaluate((element) => {
		Object.assign(element.style, {
			position: 'fixed',
			right: '12px',
			bottom: '12px',
			width: '210px',
			height: '100px',
			margin: '0',
		});
	});
	await page.locator('#first').fill('This is a tset.');
	await waitForLints(page, 1);
	expect(await clickHarperHighlight(page)).toBe(true);
	await expect(rows(page).first()).toBeFocused();
	await expectPanelsInViewport(page);
	const list = (await page.locator('.harper-container').boundingBox())!;
	const panel = (await flyout(page).boundingBox())!;
	const highlight = (await getHarperHighlights(page).first().boundingBox())!;
	expect(panel.x + panel.width).toBeLessThan(list.x);
	expect(list.y + list.height).toBeLessThan(highlight.y);
	await rows(page).nth(1).hover();
	expect(await flyoutPosition(page)).toEqual({ x: panel.x, y: panel.y });
});

test('Stacks the panels in a narrow viewport and supports dark mode and reduced motion', async ({
	page,
}) => {
	await page.setViewportSize({ width: 360, height: 640 });
	await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' });
	await openMenu(page, 'This is a tset.', 1);
	await expectPanelsInViewport(page);
	const list = (await page.locator('.harper-container').boundingBox())!;
	const panel = (await flyout(page).boundingBox())!;
	expect(panel.y).toBeGreaterThanOrEqual(list.y + list.height);
	await rows(page).nth(1).hover();
	expect(await flyoutPosition(page)).toEqual({ x: panel.x, y: panel.y });
	await expect(page.locator('.harper-container')).toHaveCSS('background-color', 'rgb(13, 17, 23)');
	await expect(page.locator('.harper-popup')).toHaveCSS('animation-name', 'none');
});

test('The activation key opens and renders without waiting for another lint update', async ({
	page,
	context,
	browserName,
}) => {
	test.skip(browserName === 'firefox', 'Firefox does not reliably expose its background context.');
	const background = await getBackground(context);
	await background.evaluate(async () => {
		await chrome.storage.local.set({ activationKey: 'shift' });
	});
	await page.goto(URL);
	await page.locator('#first').fill(TEXT);
	await waitForLints(page, 2);
	await page.locator('#first').evaluate((field: HTMLTextAreaElement) => {
		field.setSelectionRange(24, 24);
		field.focus();
	});
	await page.keyboard.press('Shift');
	await page.waitForTimeout(50);
	await page.keyboard.press('Shift');
	await expect(rows(page)).toHaveText(['could have›']);
	await expect(rows(page).first()).toBeFocused();
});

test('Editing the source closes the popup even when the new lint count is unchanged', async ({
	page,
}) => {
	await openMenu(page);
	await page.locator('#first').fill('She is an teacher. I could of gone.');
	await expect(rows(page)).toHaveCount(0);
	await waitForLints(page, 2);
	expect(await clickHarperHighlight(page)).toBe(true);
	await expect(rows(page).first()).toBeFocused();
});

/** Find render-tree nodes without mounting hooks; useful for uncommon engine outputs. */
function nodesWithClass(
	tree: ReturnType<typeof SuggestionBox>,
	className: string,
): ReturnType<typeof SuggestionBox>[] {
	const own = String(tree.properties.className ?? '')
		.split(' ')
		.includes(className)
		? [tree]
		: [];
	return own.concat(
		tree.children.flatMap((child) =>
			child.type === 'VirtualNode'
				? nodesWithClass(child as ReturnType<typeof SuggestionBox>, className)
				: [],
		),
	);
}

function sampleBox(suggestions: IgnorableLintBox['lint']['suggestions']): IgnorableLintBox {
	return {
		x: 20,
		y: 20,
		width: 40,
		height: 20,
		source: {} as HTMLElement,
		rule: 'ExampleRule',
		applySuggestion: () => {},
		lint: {
			span: { start: 0, end: 4 },
			message_html: 'A <code>message</code>.',
			problem_text: 'word',
			lint_kind: 'Spelling',
			lint_kind_pretty: 'Spelling',
			suggestions,
			context_hash: 'test',
			source: 'word',
		},
	};
}

function renderSample(box: IgnorableLintBox) {
	return SuggestionBox({
		box,
		selectedIndex: 0,
		onSelect: () => {},
		actions: {},
		hint: 'A stable hint.',
		close: () => {},
	});
}

test('A lint with no replacements still has a detail row and no unsafe apply action', async () => {
	const box = sampleBox([]);
	box.applySuggestion = () => {
		throw new Error('A missing suggestion must not be applied');
	};
	const tree = renderSample(box);
	expect(nodesWithClass(tree, 'harper-replace')).toHaveLength(0);
	expect(nodesWithClass(tree, 'harper-row')[0].properties.title).toBe('No replacements for "word"');
	nodesWithClass(tree, 'harper-row')[0].properties.onclick();
	expect(nodesWithClass(tree, 'harper-dismiss')).toHaveLength(0);
	expect(nodesWithClass(tree, 'harper-disable')).toHaveLength(0);
	expect(
		nodesWithClass(tree, 'harper-flyout').flatMap((panel) =>
			nodesWithClass(panel, 'harper-hint-drawer'),
		),
	).toHaveLength(0);
	expect(nodesWithClass(tree, 'harper-hint-drawer')).toHaveLength(1);
});

test('Empty replacement labels preserve Remove and Insert After semantics', async () => {
	for (const [kind, label, title] of [
		[SuggestionKind.Remove, 'Remove', 'Click to remove "word"'],
		[SuggestionKind.InsertAfter, 'Insert After', 'Click to insert "" after "word"'],
	] as const) {
		const tree = renderSample(sampleBox([{ kind, replacement_text: '' }]));
		expect(nodesWithClass(tree, 'harper-row')[0].properties.title).toBe(title);
		expect(nodesWithClass(tree, 'harper-row-label')[0].children).toMatchObject([{ text: label }]);
		expect(nodesWithClass(tree, 'harper-replace')).toHaveLength(0);
	}
});

test('Row tooltips describe inserted text and empty replacements accurately', async () => {
	for (const [kind, replacement_text, title] of [
		[SuggestionKind.InsertAfter, ' after', 'Click to insert " after" after "word"'],
		[SuggestionKind.Replace, '', 'Click to replace "word" with ""'],
	] as const) {
		const tree = renderSample(sampleBox([{ kind, replacement_text }]));
		expect(nodesWithClass(tree, 'harper-row')[0].properties.title).toBe(title);
	}
});

test('Every replacement alternative is rendered, not just the first', async () => {
	const tree = renderSample(
		sampleBox(
			['receive', 'received', 'recipe'].map((replacement_text) => ({
				kind: SuggestionKind.Replace,
				replacement_text,
			})),
		),
	);
	expect(nodesWithClass(tree, 'harper-row').map((node) => node.properties.title)).toEqual([
		'Click to replace "word" with "receive"',
		'Click to replace "word" with "received"',
		'Click to replace "word" with "recipe"',
	]);
	expect(nodesWithClass(tree, 'harper-replace')).toHaveLength(0);
});

test('Flyout callbacks use the displayed lint regardless of the selected alternative', async () => {
	const box = sampleBox([
		{ kind: SuggestionKind.Replace, replacement_text: 'first' },
		{ kind: SuggestionKind.Replace, replacement_text: 'second' },
	]);
	const calls: unknown[] = [];
	box.ignoreLint = async () => {
		calls.push('dismiss');
	};
	const tree = SuggestionBox({
		box,
		selectedIndex: 1,
		onSelect: () => {},
		hint: null,
		close: () => {
			calls.push('close');
		},
		actions: {
			reportError: async (lint, rule) => {
				calls.push([lint, rule]);
			},
			addToUserDictionary: async (words) => {
				calls.push(words);
			},
			setRuleEnabled: async (rule, enabled) => {
				calls.push([rule, enabled]);
			},
		},
	});
	nodesWithClass(tree, 'harper-report-link')[0].properties.onclick();
	nodesWithClass(tree, 'harper-disable')[0].properties.onclick();
	nodesWithClass(tree, 'harper-dismiss')[0].properties.onclick();
	nodesWithClass(tree, 'harper-btn')
		.find((node) => node.properties.title === 'Add word to user dictionary')!
		.properties.onclick();
	await new Promise((resolve) => setTimeout(resolve, 0));
	expect(calls).toContainEqual([box.lint, 'ExampleRule']);
	expect(calls).toContainEqual(['ExampleRule', false]);
	expect(calls).toContainEqual(['word']);
	expect(calls).toContain('dismiss');
	expect(calls.filter((call) => call === 'close')).toHaveLength(3);
});

/** Synthetic lint occurrences for testing popup state independently of engine output. */
interface TestLint {
	rule: string;
	start: number;
	end: number;
	suggestions: string[];
	secondField?: boolean;
}

declare global {
	interface Window {
		PopupHarness: typeof PopupHandler;
		HarperTestEnums: { SuggestionKind: typeof SuggestionKind };
		popupTest: { handler: PopupHandler; boxes: IgnorableLintBox[]; calls: unknown[][] };
	}
}

/**
 * Bundle the real handler without a production test API or WASM dependency.
 * Only the SDK's suggestion enum is needed; supply it from the test process so
 * concurrent workspace rebuilds cannot interrupt resolution of generated harper.js files.
 */
async function bundlePopupHarness(): Promise<string> {
	const result = await build({
		configFile: false,
		publicDir: false,
		logLevel: 'silent',
		build: {
			write: false,
			minify: false,
			lib: {
				entry: fileURLToPath(
					new NodeURL('../../lint-framework/src/lint/PopupHandler.ts', import.meta.url),
				),
				name: 'PopupHarness',
				formats: ['iife'],
			},
			rollupOptions: {
				external: ['harper.js'],
				output: { globals: { 'harper.js': 'HarperTestEnums' } },
			},
		},
	});
	const output = Array.isArray(result) ? result[0] : result;
	if (!('output' in output)) throw new Error('Expected an in-memory popup bundle');
	const chunk = output.output.find((item) => item.type === 'chunk');
	if (!chunk || chunk.type !== 'chunk') throw new Error('Popup bundle contained no JavaScript');
	return chunk.code;
}

/** Mount real popup state/rendering with deterministic lint geometry and recorded consumer actions. */
async function openHarness(page: Page, script: string, lints: TestLint[], clicked = 0) {
	await page.goto(URL);
	await page.evaluate((kind) => {
		window.HarperTestEnums = { SuggestionKind: kind };
	}, SuggestionKind);
	await page.addScriptTag({ content: script });
	await page.evaluate(
		({ lints, clicked, replaceKind }) => {
			const calls: unknown[][] = [];
			const handler = new window.PopupHarness({
				openOptions: async () => {
					calls.push(['settings']);
				},
			});
			const boxes = lints.map((lint, index): IgnorableLintBox => {
				const source = document.querySelector<HTMLTextAreaElement>(
					lint.secondField ? '#second' : '#first',
				)!;
				source.value = 'This is a sentence with several ordinary words.';
				const rect = source.getBoundingClientRect();
				return {
					x: rect.x + 10 + index * 40,
					y: rect.y + 10,
					width: 20,
					height: 20,
					source,
					rule: lint.rule,
					lint: {
						span: { start: lint.start, end: lint.end },
						context_hash: lint.rule,
						source: source.value,
						problem_text: lint.rule,
						message_html: `<p>${lint.rule} message</p>`,
						lint_kind: 'Spelling',
						lint_kind_pretty: 'Spelling',
						suggestions: lint.suggestions.map((replacement_text) => ({
							kind: replaceKind,
							replacement_text,
						})),
					},
					applySuggestion: (suggestion) => {
						calls.push(['apply', lint.rule, suggestion.replacement_text]);
					},
					ignoreLint: async () => {
						calls.push(['dismiss', lint.rule]);
					},
				};
			});
			window.popupTest = { handler, boxes, calls };
			handler.updateLintBoxes(boxes);
			const anchor = boxes[clicked];
			(anchor.source as HTMLTextAreaElement).focus();
			anchor.source.dispatchEvent(
				new PointerEvent('pointerdown', {
					bubbles: true,
					composed: true,
					clientX: anchor.x + 1,
					clientY: anchor.y + 1,
				}),
			);
		},
		{ lints, clicked, replaceKind: SuggestionKind.Replace },
	);
	await expect(rows(page).first()).toBeFocused();
}

browserTest.describe('Single-lint popup state', () => {
	let script: string;
	browserTest.beforeAll(async () => {
		script = await bundlePopupHarness();
	});
	const anchor: TestLint = { rule: 'Anchor', start: 8, end: 14, suggestions: ['first', 'second'] };

	browserTest('Go to Harper settings invokes the settings callback', async ({ page }) => {
		await openHarness(page, script, [anchor]);
		const settings = page.getByRole('button', { name: 'Go to Harper settings', exact: true });
		await expect(settings).toHaveText('Go to Harper settings');
		await expect(settings).toHaveAttribute('title', 'Click to go to Harper settings');
		await settings.click();
		await expect.poll(() => page.evaluate(() => window.popupTest.calls)).toEqual([['settings']]);
	});

	for (const [name, occurrence] of [
		['another span', { ...anchor, start: 28, end: 34 }],
		['another editor', { ...anchor, secondField: true }],
	] as const) {
		browserTest(`Does not confuse identical hashes in ${name}`, async ({ page }) => {
			await openHarness(page, script, [occurrence, anchor], 1);
			await page.keyboard.press('ArrowDown');
			const before = (await page.locator('.harper-container').boundingBox())!;
			await page.evaluate(() => {
				const { handler, boxes } = window.popupTest;
				handler.updateLintBoxes([{ ...boxes[1], y: boxes[1].y + 40 }, boxes[0]]);
			});
			await expect(rows(page).last()).toBeFocused();
			await expect
				.poll(async () => (await page.locator('.harper-container').boundingBox())?.y)
				.toBeCloseTo(before.y + 40, 0);
			// Removing the selected occurrence must not retarget the remaining identical hash.
			await page.evaluate(() => {
				const { handler, boxes } = window.popupTest;
				handler.updateLintBoxes([boxes[0]]);
			});
			await expect(rows(page)).toHaveCount(0);
		});
	}

	for (const field of ['source', 'context_hash'] as const) {
		browserTest(`Closes when the lint's ${field} changes`, async ({ page }) => {
			await openHarness(page, script, [anchor]);
			await page.evaluate((field) => {
				const { handler, boxes } = window.popupTest;
				handler.updateLintBoxes(
					boxes.map((box) => ({ ...box, lint: { ...box.lint, [field]: 'changed' } })),
				);
			}, field);
			await expect(rows(page)).toHaveCount(0);
		});
	}

	browserTest(
		'Keeps keyboard focus on the replacement when alternatives reorder or disappear',
		async ({ page }) => {
			await openHarness(page, script, [anchor]);
			await page.keyboard.press('ArrowDown');
			await expect(rows(page).last()).toBeFocused();
			await page.evaluate(() => {
				const { handler, boxes } = window.popupTest;
				handler.updateLintBoxes(
					boxes.map((box) => ({
						...box,
						lint: {
							...box.lint,
							suggestions: [...box.lint.suggestions]
								.reverse()
								.map((suggestion) => ({ ...suggestion })),
						},
					})),
				);
			});
			await expect(rows(page)).toHaveText(['second›', 'first›']);
			await expect(rows(page).first()).toBeFocused();
			await page.evaluate(() => {
				const { handler, boxes } = window.popupTest;
				handler.updateLintBoxes(
					boxes.map((box) => ({
						...box,
						lint: {
							...box.lint,
							suggestions: box.lint.suggestions.filter(
								(suggestion) => suggestion.replacement_text === 'first',
							),
						},
					})),
				);
			});
			await expect(rows(page)).toHaveText(['first›']);
			await expect(rows(page).first()).toBeFocused();
		},
	);

	browserTest('Resets selection when another lint is clicked', async ({ page }) => {
		await openHarness(page, script, [
			anchor,
			{ rule: 'Separate', start: 28, end: 32, suggestions: ['new'] },
		]);
		await rows(page).last().hover();
		await page.evaluate(() => {
			const box = window.popupTest.boxes[1];
			box.source.dispatchEvent(
				new PointerEvent('pointerdown', {
					bubbles: true,
					composed: true,
					clientX: box.x + 1,
					clientY: box.y + 1,
				}),
			);
		});
		await expect(rows(page)).toHaveText(['new›']);
		await expect(rows(page).first()).toBeFocused();
	});

	browserTest(
		'A lint without replacements retains non-applying details and actions',
		async ({ page }) => {
			await openHarness(page, script, [{ ...anchor, suggestions: [] }]);
			await expect(rows(page)).toHaveText(['No replacements›']);
			await expect(flyout(page).locator('.harper-replace')).toHaveCount(0);
			await page.keyboard.press('Enter');
			expect(await page.evaluate(() => window.popupTest.calls)).toEqual([]);
			await page.keyboard.press('ArrowRight');
			await expect(flyout(page).getByTitle('Ignore this lint')).toBeFocused();
			await page.keyboard.press('Enter');
			await expect
				.poll(() => page.evaluate(() => window.popupTest.calls))
				.toEqual([['dismiss', 'Anchor']]);
		},
	);

	browserTest(
		'ArrowRight leaves focus on the row when the flyout has no actions',
		async ({ page }) => {
			await openHarness(page, script, [anchor]);
			await page.evaluate(() => {
				const { handler, boxes } = window.popupTest;
				handler.updateLintBoxes(boxes.map((box) => ({ ...box, ignoreLint: undefined })));
			});
			await expect(flyout(page).getByRole('button')).toHaveCount(0);
			await page.keyboard.press('ArrowRight');
			await expect(rows(page).first()).toBeFocused();
			await page.keyboard.press('Enter');
			await expect
				.poll(() => page.evaluate(() => window.popupTest.calls))
				.toEqual([['apply', 'Anchor', 'first']]);
			await expect(rows(page)).toHaveCount(0);
		},
	);

	browserTest('A long replacement list scrolls without moving its flyout', async ({ page }) => {
		await page.emulateMedia({ reducedMotion: 'reduce' });
		await openHarness(page, script, [
			{ ...anchor, suggestions: Array.from({ length: 20 }, (_, i) => `choice ${i}`) },
		]);
		const positionBefore = await flyoutPosition(page);
		await page.keyboard.press('ArrowUp');
		await expect(rows(page).last()).toBeFocused();
		await expect
			.poll(() => page.locator('.harper-container').evaluate((node) => node.scrollTop))
			.toBeGreaterThan(0);
		await expect(flyout(page).getByTitle('Ignore this lint')).toBeVisible();
		expect(await flyoutPosition(page)).toEqual(positionBefore);
		await page.locator('.harper-container').evaluate((node) => {
			node.scrollTop = 0;
		});
		expect(await flyoutPosition(page)).toEqual(positionBefore);
		await expect(rows(page).last()).toHaveAttribute('aria-expanded', 'true');
		await expectPanelsInViewport(page);
	});
});
