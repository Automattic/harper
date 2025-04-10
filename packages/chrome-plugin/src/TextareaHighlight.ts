import { memoize } from 'lodash-es';
import type { VNode } from 'virtual-dom';
import createElement from 'virtual-dom/create-element';
import diff from 'virtual-dom/diff';
import h from 'virtual-dom/h';
import patch from 'virtual-dom/patch';
import { type LintBox, isPointInBox } from './Box';
import SuggestionBox from './SuggestionBox';
import Textbox from './Textbox';
import lintKindColor from './lintKindColor';
import type { UnpackedLint } from './unpackLint';

async function lint(text: string): Promise<UnpackedLint[]> {
	console.log('Hitting service worker.');
	return await chrome.runtime.sendMessage({ text });
}

const memLint = memoize(lint);

const AREA_EVENTS = ['focus', 'keyup', 'paste', 'change', 'scroll'];
const PAGE_EVENTS = ['scroll', 'resize'];

export default class TextareaHighlight {
	/** The element our virtual DOM is attached to. */
	private virtualRoot: Element | undefined;
	/** The current state of the virtual DOM */
	private virtualTree: VNode | undefined;
	/** The current highlighted lints (and their location on the screen). */
	private currentLintBoxes: LintBox[] = [];
	/** The shadow DOM the `virtualRoot` is attached to. */
	private shadowHost: HTMLElement | undefined;
	/** The target `<textarea />` */
	private target: HTMLTextAreaElement;
	/** The function to be called to re-render the highlights. */
	private updateEventCallback: (() => void) | null = null;
	private pointerCallback: ((e: PointerEvent) => void) | null = null;

	/** If defined, show a popup containing information from the index.  */
	private popupLint: number;

	constructor(target: HTMLTextAreaElement) {
		this.target = target;
		this.attachEventListeners();
		this.update();
	}

	private async renderHighlights(): Promise<VNode> {
		const text = this.target.value;
		this.currentLintBoxes = await this.generateLintBoxes(text);
		const elements = [];

		for (const box of this.currentLintBoxes) {
			const boxEl = h(
				'div',
				{
					style: {
						position: 'fixed',
						left: `${box.x}px`,
						top: `${box.y}px`,
						width: `${box.width}px`,
						height: `${box.height}px`,
						zIndex: '1000',
						pointerEvents: 'none',
						borderBottom: `2px solid ${lintKindColor(box.lint.lint_kind)}`,
					},
				},
				[],
			);

			elements.push(boxEl);
		}

    let popup = this.getCurrentPopup();

		if (popup != null) {
			elements.push(
				SuggestionBox(
          popup
				),
			);
		}

		return h('div', {}, elements);
	}

  private getCurrentPopup(): LintBox | undefined{
    if (this.popupLint){
      return this.currentLintBoxes.at(this.popupLint)
    }
  }

	private async update() {
		console.log('update');
		const tree = await this.renderHighlights();

		if (!this.virtualRoot || !this.virtualTree) {
			this.shadowHost = document.createElement('div');
			const shadow = this.shadowHost.attachShadow({ mode: 'closed' });
			this.virtualRoot = createElement(tree);
			document.body.appendChild(this.shadowHost);
			shadow.appendChild(this.virtualRoot);
		} else {
			const patches = diff(this.virtualTree, tree);
			this.virtualRoot = patch(this.virtualRoot, patches);
		}
		this.virtualTree = tree;
	}

	private async generateLintBoxes(text: string): Promise<LintBox[]> {
		const textbox = new Textbox(this.target);
		const lints = await memLint(text);
		let boxes = lints.flatMap((l) => textbox.computeBoxes(l));

    // Make sure we re-render on the update
    boxes.forEach((box) => {
      let previousCallback = box.applySuggestion;
      box.applySuggestion = (sug) => {
        previousCallback(sug);
        this.update();
        this.popupLint = undefined;
      }
    })

		return boxes;
	}

	private onPointerDown(e: PointerEvent) {
		console.log('pointerdown');
		console.log([e.x, e.y]);
		console.log(this.currentLintBoxes);

		for (const box of this.currentLintBoxes) {
			if (isPointInBox([e.x, e.y], box)) {
				console.log('hit');
				this.popupLint = box;
				this.update();
				return;
			}
		}
	}

	private attachEventListeners() {
		this.updateEventCallback = () => {
			this.update();
		};

		this.pointerCallback = (e: PointerEvent) => {
			this.onPointerDown(e);
		};

		this.target.addEventListener('pointerdown', this.pointerCallback);

		for (const event of AREA_EVENTS) {
			this.target.addEventListener(event, this.updateEventCallback);
		}

		for (const event of PAGE_EVENTS) {
			document.addEventListener(event, this.updateEventCallback);
		}
	}

	private detachEventListeners() {
		this.target.removeEventListener('pointerdown', this.pointerCallback);

		for (const event of AREA_EVENTS) {
			this.target.removeEventListener(event, this.updateEventCallback);
		}

		for (const event of PAGE_EVENTS) {
			document.removeEventListener(event, this.updateEventCallback);
		}

		this.pointerCallback = null;
		this.updateEventCallback = null;
	}

	public detach() {
		this.virtualRoot?.remove();
		this.detachEventListeners();
	}
}
