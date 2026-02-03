import {
	getLexicalEditor,
	testBasicSuggestionRichText,
	testCanIgnoreRichTextSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://playground.lexical.dev/';

testBasicSuggestionRichText(TEST_PAGE_URL, getLexicalEditor);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getLexicalEditor);
testCanIgnoreRichTextSuggestion(TEST_PAGE_URL, getLexicalEditor);
