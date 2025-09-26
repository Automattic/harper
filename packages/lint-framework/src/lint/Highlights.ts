import Bowser from 'bowser';
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
import lintKindColor, { type LintKind } from './lintKindColor';
import RenderBox from './RenderBox';
import type SourceElement from './SourceElement';
import type { UnpackedLint } from './unpackLint';

/** A class that renders highlights to a page and nothing else. Uses a virtual DOM to minimize jitter. */
export default class Highlights {
	renderBoxes: Map<SourceElement, RenderBox>;
	highlights: Map<LintKind, Highlight> | null;

	constructor() {
		this.renderBoxes = new Map();
		this.highlights = supportsCustomHighlights() ? new Map() : null;
	}

	/** Used for CSS highlight API */
	private insertHighlightStyle(tag: string, lint: UnpackedLint) {
		const color = lintKindColor(lint.lint_kind);
		const textDecor = `underline ${color} solid 2px`;
		const backgroundColor = `${color}22`;

		const styleId = `harper-highlight-style-${lint.lint_kind}`;
		if (document.getElementById(styleId)) return;

		const style = document.createElement('style');
		style.id = styleId;
		style.textContent = `
      ::highlight(${tag}) {
        text-decoration: ${textDecor};
        background-color: ${backgroundColor};
      }
    `;
		document.head.appendChild(style);
	}

	public renderLintBoxes(boxes: LintBox[]) {
		// Sort the lint boxes based on their source, so we can render them all together.
		const sourceToBoxes: Map<SourceElement, { boxes: LintBox[]; cpa: DOMRect | null }> = new Map();

		// Clear old highlights if they exist
		if (this.highlights) {
			for (const [_, highlight] of this.highlights) {
				highlight.clear();
			}
		}

		for (const box of boxes) {
			if (box.range && this.highlights != null) {
				let highlight = this.highlights.get(box.lint.lint_kind);

				if (highlight != null) {
					highlight.add(box.range);
				} else {
					highlight = new Highlight();
					const tag = `harper-${box.lint.lint_kind}`;
					CSS.highlights.set(tag, highlight);
					this.insertHighlightStyle(tag, box.lint);
					highlight.add(box.range);
					this.highlights.set(box.lint.lint_kind, highlight);
				}

				continue;
			}

			let renderBox = this.renderBoxes.get(box.source);

			if (renderBox == null) {
				renderBox = new RenderBox(this.computeRenderTarget(box.source));
				this.renderBoxes.set(box.source, renderBox);
			}

			const value = sourceToBoxes.get(box.source);
			const icr = getInitialContainingRect(renderBox.getShadowHost());

			const parent = renderBox.getShadowHost().offsetParent;
			let cpa = null;

			if (parent != null && parent != document.body && parent.shadowRoot == null) {
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
				hostStyle.width = '0px';
				hostStyle.height = '0px';
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
	private computeRenderTarget(el: SourceElement): HTMLElement {
		if (el.parentElement?.classList.contains('ProseMirror')) {
			return el.parentElement.parentElement!;
		}

		const queries = [
			getNotionRoot,
			getGhostRoot,
			getDraftRoot,
			getPMRoot,
			getCMRoot,
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
				return root.parentElement!;
			}
		}

		return el.parentElement!;
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
 */
function isContainingBlock(el: Element): boolean {
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

export function supportsCustomHighlights(ua = navigator.userAgent) {
	const root = globalThis.document?.documentElement;
	const disableFlag =
		root?.getAttribute?.('data-harper-disable-css-highlights') === 'true' ||
		root?.dataset?.harperDisableCssHighlights === 'true';
	if (disableFlag) {
		return false;
	}
	const isAutomated = globalThis.navigator?.webdriver === true;
	if (isAutomated) {
		return false;
	}
	const parser = Bowser.getParser(ua);
	const isFirefox = parser.getBrowserName(true) === 'firefox';
	if (isFirefox) return false;
	if (!('CSS' in window) || typeof CSS.supports !== 'function') return false;
	const supportsSelector = CSS.supports('selector(::highlight(__x))');
	const reg = CSS?.highlights as any;
	const hasRegistry =
		!!reg && ['get', 'set', 'has', 'delete', 'clear'].every((m) => typeof reg[m] === 'function');
	const hasCtor = typeof window.Highlight === 'function';
	let canRegister = false;
	if (hasRegistry && hasCtor) {
		try {
			const h = new Highlight();
			CSS.highlights.set('__probe__', h);
			canRegister = CSS.highlights.has('__probe__');
			CSS.highlights.delete('__probe__');
		} catch {}
	}
	return supportsSelector && hasRegistry && hasCtor && canRegister;
}
