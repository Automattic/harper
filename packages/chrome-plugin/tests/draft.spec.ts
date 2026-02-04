import type { Locator } from '@playwright/test';
import {
	getDraftEditor,
	testBasicSuggestionRichText,
	testCanIgnoreRichTextSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://draftjs.org/';

async function setup(editor: Locator) {
	await editor.scrollIntoViewIfNeeded();
	await editor.click();
}

testBasicSuggestionRichText(TEST_PAGE_URL, getDraftEditor, setup);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getDraftEditor, setup);
testCanIgnoreRichTextSuggestion(TEST_PAGE_URL, getDraftEditor, setup);
