import {
	getProseMirrorEditor,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/prosemirror.html';

testBasicSuggestion(TEST_PAGE_URL, getProseMirrorEditor);
testCanIgnoreSuggestion(TEST_PAGE_URL, getProseMirrorEditor);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getProseMirrorEditor);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getProseMirrorEditor);
