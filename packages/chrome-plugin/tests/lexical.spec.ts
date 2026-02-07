import {
	getLexicalEditor,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://playground.lexical.dev/';

testBasicSuggestion(TEST_PAGE_URL, getLexicalEditor);
testCanIgnoreSuggestion(TEST_PAGE_URL, getLexicalEditor);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getLexicalEditor);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getLexicalEditor);
