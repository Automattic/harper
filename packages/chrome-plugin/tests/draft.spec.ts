import type { Locator, Page } from '@playwright/test';
import {
	getDraftEditor,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://draftjs.org/';

async function setup(_page: Page, editor: Locator) {
	await editor.scrollIntoViewIfNeeded();
	await editor.click();
}

testBasicSuggestion(TEST_PAGE_URL, getDraftEditor, setup);
testCanIgnoreSuggestion(TEST_PAGE_URL, getDraftEditor, setup);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getDraftEditor, setup);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getDraftEditor, setup);
