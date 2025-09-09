import type { VNode } from 'virtual-dom';
import createElement from 'virtual-dom/create-element';
import diff from 'virtual-dom/diff';
import patch from 'virtual-dom/patch';

/** Wraps `virtual-dom` to create a box that is unaffected by the style of the rest of the page. */
export default class RenderBox {
	/** The element our virtual DOM is attached to. */
	private virtualRoot: Element | undefined;
	/** The current state of the virtual DOM */
	private virtualTree: VNode | undefined;
	/** The shadow DOM the `virtualRoot` is attached to. */
	private shadowHost: HTMLElement;

	constructor(parent: Node) {
		this.shadowHost = document.createElement('div');
		parent.appendChild(this.shadowHost);
	}

	/** Render to the box. */
	public render(node: VNode) {
		if (!this.virtualRoot || !this.virtualTree) {
			this.virtualRoot = createElement(node);
			const shadow = this.shadowHost.attachShadow({ mode: 'open' });
			shadow.appendChild(this.virtualRoot);
		} else {
			const patches = diff(this.virtualTree, node);
			this.virtualRoot = patch(this.virtualRoot, patches);
		}
		this.virtualTree = node;
	}

	/** Remove the box from the DOM. */
	public remove() {
		try {
			this.shadowHost.outerHTML = this.shadowHost.outerHTML;
		} catch (e) {
			console.error(e);
		}
		this.virtualRoot = undefined;
		this.virtualTree = undefined;
	}

	public getShadowHost(): HTMLElement {
		return this.shadowHost;
	}
}
