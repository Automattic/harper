import {testPageHasNHighlights} from "./testUtils"

const TEST_PAGE_URL = 'http://localhost:8081/quill_simple.html';

testPageHasNHighlights(TEST_PAGE_URL, 1)
