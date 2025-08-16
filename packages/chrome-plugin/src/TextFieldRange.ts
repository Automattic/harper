import { boxesOverlap, domRectToBox } from './Box';

/** A version of the `Range` object that works for `<textarea />` and `<input />` elements. */
export default class TextFieldRange {
	field: HTMLTextAreaElement | HTMLInputElement;
	mirror: HTMLElement | null;
	mirrorTextNode: Text;
	startOffset: number;
	endOffset: number;

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
		this.mirror = null;
		this._createMirror();
	}

	/**
	 * Creates an off-screen mirror element that mimics the field's styles and positions it exactly over the field.
	 */
	private _createMirror(): void {
		this.mirror = document.createElement('div');
		this.mirror.id = 'textfield-mirror';

		// Copy necessary computed styles from the field (affecting text layout)
		const computed: CSSStyleDeclaration = window.getComputedStyle(this.field);
		// The properties below help ensure the mirror text has the same layout as the actual text.
		const propertiesToCopy: Array<keyof CSSStyleDeclaration> = [
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
			this.mirror!.style[prop] = computed[prop];
		});

		if (this.field instanceof HTMLTextAreaElement) {
			this.mirror.style.overflowX = 'auto';
			this.mirror.style.overflowY = 'auto';
		}

		// Compute the absolute position of the field.
		const fieldRect = this.field.getBoundingClientRect();
		const scrollTop = window.scrollY || document.documentElement.scrollTop;
		const scrollLeft = window.scrollX || document.documentElement.scrollLeft;

		// Position the mirror exactly over the field.
		Object.assign(this.mirror.style, {
			top: `${fieldRect.top + scrollTop}px`,
			left: `${fieldRect.left + scrollLeft}px`,
			width: `${fieldRect.width}px`,
			height: `${fieldRect.height}px`,
			boxSizing: 'border-box',
			// For a textarea, use "pre-wrap" (so line-breaks are preserved); for a single‑line input, use "pre"
			whiteSpace: this.field.tagName.toLowerCase() === 'textarea' ? 'pre-wrap' : 'pre',
			wordWrap: 'break-word',
			visibility: 'hidden',
			position: 'absolute',
			pointerEvents: 'none',
		});

		// Create the text node that will mirror the field's text.
		this.mirrorTextNode = document.createTextNode('');
		this.mirror.appendChild(this.mirrorTextNode);

		// Needed for the scroll to work.
		this._updateMirrorText();

		// Append the mirror element to the document body.
		document.body.appendChild(this.mirror);

		this.mirror.scrollTo({
			top: this.field.scrollTop,
			left: this.field.scrollLeft,
			behavior: 'instant',
		});
	}

	/**
	 * Updates the mirror's text node with the current value of the field.
	 */
	private _updateMirrorText(): void {
		this.mirrorTextNode.nodeValue = this.field.value;
	}

	/**
	 * Returns an array of DOMRect objects corresponding to the range's visual segments.
	 * This mimics the native Range.getClientRects() method.
	 * @returns {DOMRect[]} An array of DOMRect objects.
	 */
	getClientRects(): DOMRect[] {
		this._updateMirrorText();

		const range = document.createRange();
		range.setStart(this.mirrorTextNode, this.startOffset);
		range.setEnd(this.mirrorTextNode, this.endOffset);

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
		this._updateMirrorText();
		if (this.mirror == null) {
			return null;
		}

		return this.mirror.getBoundingClientRect();
	}

	/**
	 * Detaches (removes) the mirror element from the document.
	 */
	detach(): void {
		if (this.mirror?.parentNode) {
			this.mirror.parentNode.removeChild(this.mirror);
			this.mirror = null;
		}
	}
}
