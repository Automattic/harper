import { fileURLToPath } from 'node:url';
import rehypeHighlight from 'rehype-highlight';

/**
 * Escape code text after highlighting, before mdsvex serializes it as a Svelte template.
 * Keep highlight spans intact and leave prose, embedded HTML, and Svelte expressions alone.
 * The highlighter is disabled below without mdsvex's early code escaping, so entities such
 * as `&lt;` and literal braces are escaped exactly once and retain their original text.
 */
function rehypeEscapeHighlightedCode() {
	function visit(node, inCodeBlock = false) {
		const isCodeBlock = inCodeBlock || (node.type === 'element' && node.tagName === 'pre');
		if (isCodeBlock && node.type === 'text') {
			const entities = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '{': '&#123;', '}': '&#125;' };
			node.value = node.value.replace(/[&<>{}]/g, (character) => entities[character]);
		}
		for (const child of node.children ?? []) visit(child, isCodeBlock);
	}
	return (tree) => visit(tree);
}

export default {
	extensions: ['.md'],
	layout: fileURLToPath(new URL('./src/lib/docs/MarkdownLayout.svelte', import.meta.url)),
	smartypants: false,
	// `highlight: false` escapes code before rehype can highlight it. Disable only the
	// highlighter instead, and let the final rehype plugin perform code escaping once.
	highlight: { highlighter: false },
	rehypePlugins: [
		[rehypeHighlight, { detect: false, ignoreMissing: true }],
		rehypeEscapeHighlightedCode,
	],
};
