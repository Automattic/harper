import {
	getSlateEditor,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/slate.html';

testBasicSuggestion(TEST_PAGE_URL, getSlateEditor);
testCanIgnoreSuggestion(TEST_PAGE_URL, getSlateEditor);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getSlateEditor);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getSlateEditor);
