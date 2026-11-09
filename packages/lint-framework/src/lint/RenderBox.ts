import type { VNode } from 'virtual-dom';
import createElement from 'virtual-dom/create-element';
import diff from 'virtual-dom/diff';
import patch from 'virtual-dom/patch';

type ParentResolver = () => Node;

/** The id of the invisible `position: fixed` marker used by `RenderBox.measureFixedFrame`. */
export const FIXED_FRAME_MARKER_ID = 'harper-fixed-frame';

/** The on-screen origin and scale of `position: fixed` content inside a render box. */
export type FixedFrame = { x: number; y: number; scaleX: number; scaleY: number };

/** Wraps `virtual-dom` to create a box that is unaffected by the style of the rest of the page. */
export default class RenderBox {
	/** The node we reattach into if a host is removed by editor-managed DOM updates. */
	private parentResolver: ParentResolver;
	/** The element our virtual DOM is attached to. */
	private virtualRoot: Element | undefined;
	/** The current state of the virtual DOM */
	private virtualTree: VNode | undefined;
	/** The shadow DOM the `virtualRoot` is attached to. */
	private shadowHost: HTMLElement;

	constructor(parent: Node | ParentResolver) {
		this.parentResolver = typeof parent === 'function' ? parent : () => parent;
		this.shadowHost = document.createElement('harper-render-box');
		this.parentResolver().appendChild(this.shadowHost);
	}

	/** Render to the box. */
	public render(node: VNode) {
		if (!this.shadowHost.isConnected) {
			this.parentResolver().appendChild(this.shadowHost);
		}

		if (!this.virtualRoot || !this.virtualTree) {
			this.virtualRoot = createElement(node);
			this.getShadowRoot().appendChild(this.virtualRoot);
		} else {
			const patches = diff(this.virtualTree, node);
			this.virtualRoot = patch(this.virtualRoot, patches);
		}
		this.virtualTree = node;
	}

	/**
	 * Measure where the rendered frame marker (see `FIXED_FRAME_MARKER_ID`), a `position: fixed`
	 * element placed at (0, 0) and 100px wide, actually lands on screen and how much it is
	 * scaled. Ancestors with CSS `zoom`, `transform`, `scale` and similar properties change
	 * both, so highlights have to be mapped through this frame. Returns `null` before the
	 * marker has been rendered.
	 */
	public measureFixedFrame(): FixedFrame | null {
		const marker = this.shadowHost.shadowRoot?.getElementById(FIXED_FRAME_MARKER_ID);
		if (marker == null || !marker.isConnected) {
			return null;
		}

		const rect = marker.getBoundingClientRect();

		return {
			x: rect.x,
			y: rect.y,
			scaleX: rect.width > 0 ? rect.width / 100 : 1,
			scaleY: rect.height > 0 ? rect.height / 100 : 1,
		};
	}

	private getShadowRoot(): ShadowRoot {
		return this.shadowHost.shadowRoot ?? this.shadowHost.attachShadow({ mode: 'open' });
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
