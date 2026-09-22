import type { Locator, Page } from '@playwright/test';
import {
	getDraftEditor,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/draft.html';

async function setup(_page: Page, editor: Locator) {
	await editor.scrollIntoViewIfNeeded();
	await editor.click();
}

testBasicSuggestion(TEST_PAGE_URL, getDraftEditor, setup);
testCanIgnoreSuggestion(TEST_PAGE_URL, getDraftEditor, setup);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getDraftEditor, setup);
