import type { VNode } from 'virtual-dom';
import createElement from 'virtual-dom/create-element';
import diff from 'virtual-dom/diff';
import h from 'virtual-dom/h';
import patch from 'virtual-dom/patch';
import { type LintBox, isBoxInScreen } from './Box';
import RenderBox from './RenderBox';
import lintKindColor from './lintKindColor';

/** A class that renders highlights to a page and nothing else. Uses a virtual DOM to minimize jitter. */
export default class Highlights {
	renderBoxes: Map<HTMLElement, RenderBox>;

	constructor() {
		this.renderBoxes = new Map();
	}

	public renderLintBoxes(boxes: LintBox[]) {
		// Sort the lint boxes based on their source, so we can render them all together.
		const sourceToBoxes: Map<HTMLElement, LintBox[]> = new Map();

		for (const box of boxes) {
			const value = sourceToBoxes.get(box.source);

			if (value == null) {
				sourceToBoxes.set(box.source, [box]);
			} else {
				sourceToBoxes.set(box.source, [...value, box]);
			}
		}

		const updated = new Set();

		for (const [source, boxes] of sourceToBoxes.entries()) {
			let renderBox = this.renderBoxes.get(source);

			if (renderBox == null) {
				renderBox = new RenderBox(source.parentElement);
				this.renderBoxes.set(source, renderBox);
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

	/** Remove render boxes for sources that aren't attached any longer. */
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
						left: `${box.x}px`,
						top: `${box.y}px`,
						width: `${box.width}px`,
						height: `${box.height}px`,
						pointerEvents: 'none',
						borderBottom: `2px solid ${lintKindColor(box.lint.lint_kind)}`,
					},
				},
				[],
			);

			elements.push(boxEl);
		}

		return h('div', {}, elements);
	}
}
