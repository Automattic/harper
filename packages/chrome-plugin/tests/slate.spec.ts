import {
	getSlateEditor,
	testBasicSuggestionRichText,
	testCanIgnoreRichTextSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://slatejs.org';

testBasicSuggestionRichText(TEST_PAGE_URL, getSlateEditor);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getSlateEditor);
testCanIgnoreRichTextSuggestion(TEST_PAGE_URL, getSlateEditor);
