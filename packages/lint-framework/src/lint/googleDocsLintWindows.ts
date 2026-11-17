import { isGoogleDocsTarget } from './computeLintBoxes/googleDocsUtilities';
import { leafNodes } from './domUtils';
import type { UnpackedSpan } from './unpackLint';

export const MAX_LINT_TEXT_LENGTH = 120000;
const MAX_VISIBLE_PARAGRAPHS = 32;

/**
 * Select complete paragraphs from the full native projection, not from visual
 * labels. Projection offsets and Harper JS spans both count UTF-16 code units.
 * The native mirror inserts blank lines at hard paragraph/table boundaries;
 * single newlines can be soft wraps and must stay inside their paragraph.
 */
export function getGoogleDocsLintWindows(target: HTMLElement, source: string): UnpackedSpan[] {
	if (!isGoogleDocsTarget(target) || target.getAttribute('data-harper-gdocs-source') !== 'native') {
		return [];
	}
	const editor = target.closest('.kix-appview-editor');
	if (!editor) return [];
	const editorRect = editor.getBoundingClientRect();
	const windows: UnpackedSpan[] = [];
	const seen = new Set<number>();
	let traversed = 0;
	let remaining = MAX_LINT_TEXT_LENGTH;
	for (const leaf of leafNodes(target)) {
		const start = traversed;
		traversed += leaf.textContent?.length ?? 0;
		const host = leaf.parentElement?.closest('[data-harper-gdocs-native-start]');
		if (!host || !target.contains(host)) continue;
		const rect = host.getBoundingClientRect();
		if (
			rect.width <= 0 ||
			rect.height <= 0 ||
			rect.bottom <= Math.max(0, editorRect.top) ||
			rect.top >= Math.min(window.innerHeight, editorRect.bottom) ||
			rect.right <= Math.max(0, editorRect.left) ||
			rect.left >= Math.min(window.innerWidth, editorRect.right)
		) {
			continue;
		}
		let cursor = start;
		while (cursor < traversed) {
			const previousBreak = source.lastIndexOf('\n\n', cursor - 1);
			const paragraphStart = previousBreak < 0 ? 0 : previousBreak + 2;
			const nextBreak = source.indexOf('\n\n', cursor);
			const paragraphEnd = nextBreak < 0 ? source.length : nextBreak;
			const length = paragraphEnd - paragraphStart;
			if (!seen.has(paragraphStart)) {
				seen.add(paragraphStart);
				// Never clip a paragraph to fit. Later, smaller paragraphs can still fit.
				if (length > 0 && length <= remaining) {
					windows.push({ start: paragraphStart, end: paragraphEnd });
					remaining -= length;
					if (windows.length === MAX_VISIBLE_PARAGRAPHS) return windows;
				}
			}
			cursor = paragraphEnd + 2;
		}
	}
	return windows;
}
