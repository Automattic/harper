import {
	getProseMirrorEditor,
	testBasicSuggestion,
	testCanBlockRuleSuggestion,
	testCanIgnoreSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://prosemirror.net/';

testBasicSuggestion(TEST_PAGE_URL, getProseMirrorEditor);
testCanIgnoreSuggestion(TEST_PAGE_URL, getProseMirrorEditor);
testCanBlockRuleSuggestion(TEST_PAGE_URL, getProseMirrorEditor);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getProseMirrorEditor);
