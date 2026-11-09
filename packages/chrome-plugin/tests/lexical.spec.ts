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

testBasicSuggestion(TEST_PAGE_URL, getLexicalEditor, waitForLexicalReady);
testCanIgnoreSuggestion(TEST_PAGE_URL, getLexicalEditor, waitForLexicalReady);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getLexicalEditor, waitForLexicalReady);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getLexicalEditor, waitForLexicalReady);
