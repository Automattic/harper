import type { VNode } from 'virtual-dom';
import h from 'virtual-dom/h';
import type { LintBox } from './Box';
import {
	getCMRoot,
	getDraftRoot,
	getGhostRoot,
	getGutenbergRoot,
	getLexicalRoot,
	getMediumRoot,
	getNotionRoot,
	getP2Root,
	getPMRoot,
	getQuillJsRoot,
	getShredditComposerRoot,
	getSlateRoot,
	getTrixRoot,
} from './editorUtils';
import lintKindColor from './lintKindColor';
import RenderBox from './RenderBox';

/** A class that renders highlights to a page and nothing else. Uses a virtual DOM to minimize jitter. */
export default class Highlights {
	renderBoxes: Map<HTMLElement, RenderBox>;

	constructor() {
		this.renderBoxes = new Map();
	}

	public renderLintBoxes(boxes: LintBox[]) {
		// Sort the lint boxes based on their source, so we can render them all together.
		const sourceToBoxes: Map<HTMLElement, { boxes: LintBox[]; cpa: DOMRect | null }> = new Map();

		for (const box of boxes) {
			let renderBox = this.renderBoxes.get(box.source);

			if (renderBox == null) {
				renderBox = new RenderBox(this.computeRenderTarget(box.source));
				this.renderBoxes.set(box.source, renderBox);
			}

			const value = sourceToBoxes.get(box.source);
			const icr = getInitialContainingRect(renderBox.getShadowHost());

			const parent = renderBox.getShadowHost().offsetParent;
			let cpa = null;

			if (parent != null && parent != document.body) {
				cpa = parent.getBoundingClientRect();
			}

			if (cpa == null) {
				if (icr != null) {
					cpa = icr;
				}
			}

			if (value == null) {
				sourceToBoxes.set(box.source, { boxes: [box], cpa });
			} else {
				sourceToBoxes.set(box.source, { boxes: [...value.boxes, box], cpa });
			}
		}

		const updated = new Set();

		for (const [source, { boxes, cpa }] of sourceToBoxes.entries()) {
			const renderBox = this.renderBoxes.get(source)!;

			const host = renderBox.getShadowHost();
			host.id = 'harper-highlight-host';

			if (cpa != null) {
				const hostStyle = host.style;

				hostStyle.contain = 'layout';
				hostStyle.position = 'absolute';
				hostStyle.top = '0px';
				hostStyle.left = '0px';
				hostStyle.transform = `translate(${-cpa.x}px, ${-cpa.y}px)`;
				hostStyle.inset = '0';
				hostStyle.pointerEvents = 'none';
			}

			renderBox.render(this.renderTree(boxes));
			updated.add(source);
		}

		for (const [source, box] of this.renderBoxes.entries()) {
			if (!updated.has(source)) {
				box.render(h('div', {}, []));
			}
		}

		this.pruneDetachedSources();
	}

	/** Remove the render boxes for sources that aren't attached any longer. */
	private pruneDetachedSources() {
		for (const [source, box] of this.renderBoxes.entries()) {
			if (!document.contains(source)) {
				box.remove();
				this.renderBoxes.delete(source);
			}
		}
	}

	private renderTree(boxes: LintBox[]): VNode {
		const elements = [];

		for (const box of boxes) {
			const boxEl = h(
				'div',
				{
					style: {
						position: 'fixed',
						left: '0px',
						top: '0px',
						transform: `translate(${box.x}px, ${box.y}px)`,
						width: `${box.width}px`,
						height: `${box.height}px`,
						pointerEvents: 'none',
						borderBottom: `2px solid ${lintKindColor(box.lint.lint_kind)}`,
						backgroundColor: `${lintKindColor(box.lint.lint_kind)}22`,
					},
					id: 'harper-highlight',
				},
				[],
			);

			elements.push(boxEl);
		}

		return h('div', {}, elements);
	}

	/** Determines which target the render boxes should be attached to.
	 * Depends on text editor. */
	private computeRenderTarget(el: HTMLElement): HTMLElement {
		if (el.parentElement?.classList.contains('ProseMirror')) {
			return el.parentElement.parentElement;
		}

		const queries = [
			getGhostRoot,
			getDraftRoot,
			getPMRoot,
			getCMRoot,
			getNotionRoot,
			getSlateRoot,
			getMediumRoot,
			getShredditComposerRoot,
			getQuillJsRoot,
			getLexicalRoot,
			getP2Root,
			getGutenbergRoot,
			getTrixRoot,
		];

		for (const query of queries) {
			const root = query(el);
			if (root != null) {
				return root.parentElement;
			}
		}

		return el.parentElement;
	}
}

function getInitialContainingRect(el: HTMLElement): DOMRect | null {
	let node = el.parentElement;

	while (node && node.nodeType === 1) {
		if (isContainingBlock(node)) {
			return node.getBoundingClientRect();
		}
		node = node.parentElement;
	}

	return null;
}

/**
 * Determines whether a given element would form the containing block
 * for a descendant with `position: fixed`, based on CSS transforms,
 * filters, containment, container queries, will-change, and
 * content-visibility.
 *
 * Logs the element and the precise reason it qualifies.
 *
 * @param {Element} el
 * @returns {boolean}
 */
function isContainingBlock(el): boolean {
	if (!(el instanceof Element)) {
		throw new TypeError('Expected a DOM Element');
	}

	const style = window.getComputedStyle(el);

	const filter = style.getPropertyValue('filter');
	if (filter !== 'none') {
		return true;
	}

	const backdrop = style.getPropertyValue('backdrop-filter');
	if (backdrop !== 'none') {
		return true;
	}

	const transform = style.getPropertyValue('transform');
	if (transform !== 'none') {
		return true;
	}

	const perspective = style.getPropertyValue('perspective');
	if (perspective !== 'none') {
		return true;
	}

	const contain = style.getPropertyValue('contain');
	const containMatch = contain.match(/\b(layout|paint|strict|content)\b/);
	if (containMatch) {
		return true;
	}

	const willChange = style.getPropertyValue('will-change');
	if (willChange && willChange.trim() !== 'auto') {
		const declared = willChange.split(',').map((p) => p.trim());
		const triggers = ['filter', 'backdrop-filter', 'transform', 'perspective'];
		const intersection = declared.filter((p) => triggers.includes(p));
		if (intersection.length) {
			return true;
		}
	}

	const contentVis = style.getPropertyValue('content-visibility');
	if (contentVis === 'auto') {
		return true;
	}

	return false;
}
