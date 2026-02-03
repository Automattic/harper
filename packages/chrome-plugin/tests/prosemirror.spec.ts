import {
	getProseMirrorEditor,
	testBasicSuggestionRichText,
	testCanIgnoreRichTextSuggestion,
	testMultipleSuggestionsAndUndo,
} from './testUtils';

const TEST_PAGE_URL = 'https://prosemirror.net/';

testBasicSuggestionRichText(TEST_PAGE_URL, getProseMirrorEditor);
testMultipleSuggestionsAndUndo(TEST_PAGE_URL, getProseMirrorEditor);
testCanIgnoreRichTextSuggestion(TEST_PAGE_URL, getProseMirrorEditor);
