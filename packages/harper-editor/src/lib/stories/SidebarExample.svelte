<script lang="ts">
import LintSidebar from '../LintSidebar.svelte';
import type { IgnorableLintBox } from '../types.js';
import { exampleLint } from './fixtures.js';

export let populated = true;
/** Wrap the sidebar in a `.dark` ancestor to show its dark palette. */
export let dark = false;

const lintBox: IgnorableLintBox = {
	x: 0,
	y: 0,
	width: 0,
	height: 0,
	lint: exampleLint,
	source: document.createTextNode(exampleLint.source),
	rule: 'Spelling',
	applySuggestion: () => {},
	ignoreLint: async () => {},
};
</script>

<div class:dark>
	<div class="harper-editor" style="height: 450px; display: flex;">
		<LintSidebar lintBoxes={populated ? [lintBox] : []} />
	</div>
</div>
