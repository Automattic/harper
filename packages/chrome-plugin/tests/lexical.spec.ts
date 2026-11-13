import type { Page } from '@playwright/test';
import { expect } from './fixtures';
import {
	getLexicalEditor,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/lexical.html';

/** Wait for the CDN modules, editor, and undo history before sending keyboard input. */
async function waitForLexicalReady(page: Page) {
	await expect(page.locator('#status')).toHaveText('Ready', { timeout: 30000 });
	await expect(getLexicalEditor(page)).toBeEditable();
}

/** A frame can finish before Lexical imports the native arrow-key selection. */
async function waitForLexicalSelection(page: Page) {
	await expect
		.poll(
			() =>
				page.evaluate(() => {
					const fixture = window as typeof window & {
						isLexicalSelectionSynchronized?: () => boolean;
					};
					return fixture.isLexicalSelectionSynchronized?.() === true;
				}),
			{
				message: 'Lexical selection should match both native selection endpoints before Backspace',
				timeout: 5000,
			},
		)
		.toBe(true);
}

testBasicSuggestion(TEST_PAGE_URL, getLexicalEditor, waitForLexicalReady, waitForLexicalSelection);
testCanIgnoreSuggestion(TEST_PAGE_URL, getLexicalEditor, waitForLexicalReady);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getLexicalEditor, waitForLexicalReady);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getLexicalEditor, waitForLexicalReady);
