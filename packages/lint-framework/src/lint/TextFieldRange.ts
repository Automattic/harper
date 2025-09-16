import type { ConditionalKeys, WritableKeysOf } from 'type-fest';
import { boxesOverlap, domRectToBox } from './Box';

/** A version of the `Range` object that works for `<textarea />` and `<input />` elements. */
export default class TextFieldRange {
	field: HTMLTextAreaElement | HTMLInputElement;
	startOffset: number;
	endOffset: number;

	// Shared arena per field to avoid repeated layout work
	private static arenas: WeakMap<
		HTMLTextAreaElement | HTMLInputElement,
		{
			mirror: HTMLDivElement;
			text: Text;
			refs: number;
		}
	> = new WeakMap();

	private arena: { mirror: HTMLDivElement; text: Text; refs: number };

	/**
	 * Create a range-like object for a given text input field.
	 * @param field - A HTMLTextAreaElement or a HTMLInputElement (of type "text").
	 * @param startOffset - The starting character index.
	 * @param endOffset - The ending character index.
	 */
	constructor(
		field: HTMLTextAreaElement | HTMLInputElement,
		startOffset: number,
		endOffset: number,
	) {
		// In this case we assume the caller provided a text field
		if (!(field instanceof HTMLTextAreaElement || field instanceof HTMLInputElement)) {
			throw new Error('TextFieldRange expects an HTMLTextAreaElement or HTMLInputElement');
		}
		this.field = field;
		this.startOffset = startOffset;
		this.endOffset = endOffset;
		this.arena = TextFieldRange.ensureArena(this.field);
		this.arena.refs++;
	}

	/**
	 * Creates (or reuses) an off-screen mirror element that mimics the field's styles
	 * and positions it exactly over the field.
	 */
	private static ensureArena(field: HTMLTextAreaElement | HTMLInputElement): {
		mirror: HTMLDivElement;
		text: Text;
		refs: number;
	} {
		const existing = TextFieldRange.arenas.get(field);
		if (existing) return existing;

		const mirror = document.createElement('div');
		mirror.className = 'harper-textfield-mirror';

		// Copy necessary computed styles from the field (affecting text layout)
		const computed: CSSStyleDeclaration = window.getComputedStyle(field);
		const propertiesToCopy: Array<
			ConditionalKeys<Pick<CSSStyleDeclaration, WritableKeysOf<CSSStyleDeclaration>>, string>
		> = [
			'fontFamily',
			'fontSize',
			'fontWeight',
			'fontStyle',
			'letterSpacing',
			'lineHeight',
			'textTransform',
			'paddingTop',
			'paddingRight',
			'paddingBottom',
			'paddingLeft',
			'borderTopWidth',
			'borderRightWidth',
			'borderBottomWidth',
			'borderLeftWidth',
			'boxSizing',
			'overflowX',
			'overflowY',
		];

		propertiesToCopy.forEach((prop) => {
			(mirror.style as any)[prop] = (computed as any)[prop];
		});

		if (field instanceof HTMLTextAreaElement) {
			mirror.style.overflowX = 'auto';
			mirror.style.overflowY = 'auto';
		}

		// Position the mirror exactly over the field.
		TextFieldRange.positionMirror(mirror, field);

		Object.assign(mirror.style, {
			boxSizing: 'border-box',
			whiteSpace: field.tagName.toLowerCase() === 'textarea' ? 'pre-wrap' : 'pre',
			wordWrap: 'break-word',
			visibility: 'hidden',
			position: 'absolute',
			pointerEvents: 'none',
		});

		const text = document.createTextNode('');
		mirror.appendChild(text);

		// Initialize text + scroll
		text.nodeValue = field.value;
		document.body.appendChild(mirror);
		mirror.scrollTop = field.scrollTop;
		mirror.scrollLeft = field.scrollLeft;

		const arena = { mirror, text, refs: 0 } as const;
		TextFieldRange.arenas.set(field, arena);
		return arena;
	}

	private static positionMirror(
		mirror: HTMLDivElement,
		field: HTMLTextAreaElement | HTMLInputElement,
	) {
		const fieldRect = field.getBoundingClientRect();
		const scrollTop = window.scrollY || document.documentElement.scrollTop;
		const scrollLeft = window.scrollX || document.documentElement.scrollLeft;
		Object.assign(mirror.style, {
			top: `${fieldRect.top + scrollTop}px`,
			left: `${fieldRect.left + scrollLeft}px`,
			width: `${fieldRect.width}px`,
			height: `${fieldRect.height}px`,
		});
	}

	/**
	 * Updates the mirror's text node with the current value of the field.
	 */
	private syncMirror(): void {
		// Ensure text, scroll, and position reflect the current field
		this.arena.text.nodeValue = this.field.value;
		this.arena.mirror.scrollTop = this.field.scrollTop;
		this.arena.mirror.scrollLeft = this.field.scrollLeft;
		TextFieldRange.positionMirror(this.arena.mirror, this.field);
	}

	/**
	 * Returns an array of DOMRect objects corresponding to the range's visual segments.
	 * This mimics the native Range.getClientRects() method.
	 * @returns {DOMRect[]} An array of DOMRect objects.
	 */
	getClientRects(): DOMRect[] {
		this.syncMirror();

		const range = document.createRange();
		range.setStart(this.arena.text, this.startOffset);
		range.setEnd(this.arena.text, this.endOffset);

		let arr = Array.from(range.getClientRects());

		const fieldBox = domRectToBox(this.field.getBoundingClientRect());

		// Filter out rectangles that should be hidden
		arr = arr.filter((rect) => {
			const box = domRectToBox(rect);
			return boxesOverlap(box, fieldBox);
		});

		return arr;
	}

	getBoundingClientRect(): DOMRect | null {
		this.syncMirror();
		return this.arena.mirror.getBoundingClientRect();
	}

	/**
	 * Detaches (removes) the mirror element from the document.
	 */
	detach(): void {
		// Release this handle; keep the shared mirror for reuse unless the field is gone.
		this.arena.refs = Math.max(0, this.arena.refs - 1);
		// If the field is no longer in the document, clean up the arena.
		if (!document.contains(this.field)) {
			try {
				this.arena.mirror.parentNode?.removeChild(this.arena.mirror);
			} catch {}
			TextFieldRange.arenas.delete(this.field);
		}
	}
}
