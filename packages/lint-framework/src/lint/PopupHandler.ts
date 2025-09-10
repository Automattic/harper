import h from 'virtual-dom/h';
import { closestBox, type IgnorableLintBox, isPointInBox } from './Box';
import { getCaretPosition } from './editorUtils';

type ActivationKey = 'off' | 'shift' | 'control';

import RenderBox from './RenderBox';
import SuggestionBox from './SuggestionBox';

type ActivationHandler = () => void;

function monitorActivationKey(
	onActivation: ActivationHandler,
	key: string,
	interval = 300,
): () => void {
	let lastTime = 0;
	const handler = (e: KeyboardEvent) => {
		if (e.key.toLowerCase() !== key.toLowerCase()) return;
		const now = performance.now();
		const diff = now - lastTime;
		if (diff <= interval && diff > 10) onActivation();
		lastTime = now;
	};
	window.addEventListener('keydown', handler);
	return () => window.removeEventListener('keydown', handler);
}

export default class PopupHandler {
	private currentLintBoxes: IgnorableLintBox[];
	private popupLint: number | undefined;
	private renderBox: RenderBox;
	private pointerDownCallback: (e: PointerEvent) => void;
	private activationKeyListener: (() => void) | undefined;
	private readonly actions: {
		getActivationKey?: () => Promise<ActivationKey>;
		openOptions?: () => Promise<void>;
		addToUserDictionary?: (words: string[]) => Promise<void>;
	};

	constructor(actions: {
		getActivationKey?: () => Promise<ActivationKey>;
		openOptions?: () => Promise<void>;
		addToUserDictionary?: (words: string[]) => Promise<void>;
	}) {
		this.actions = actions;
		this.currentLintBoxes = [];
		this.renderBox = new RenderBox(document.body);
		this.renderBox.getShadowHost().popover = 'manual';
		this.renderBox.getShadowHost().style.pointerEvents = 'none';
		this.renderBox.getShadowHost().style.border = 'none';
		this.pointerDownCallback = (e) => {
			this.onPointerDown(e);
		};

		this.updateActivationKeyListener();
	}

	private updateActivationKeyListener() {
		if (this.activationKeyListener) {
			this.activationKeyListener();
			this.activationKeyListener = undefined;
		}

		const getKey = this.actions.getActivationKey;
		if (getKey) {
			getKey().then((key) => {
				if (key !== 'off') {
					this.activationKeyListener = monitorActivationKey(() => this.openClosestToCaret(), key);
				}
			});
		}
	}

	/** Tries to get the current caret position.
	 * If successful, opens the popup closes to it. */
	private openClosestToCaret() {
		const caretPosition = getCaretPosition();

		if (caretPosition != null) {
			const closestIdx = closestBox(caretPosition, this.currentLintBoxes);

			if (closestIdx >= 0) {
				this.popupLint = closestIdx;
			}
		}
	}

	private onPointerDown(e: PointerEvent) {
		for (let i = 0; i < this.currentLintBoxes.length; i++) {
			const box = this.currentLintBoxes[i];

			if (isPointInBox([e.x, e.y], box)) {
				this.popupLint = i;
				this.render();
				return;
			}
		}

		this.popupLint = undefined;
		this.render();
	}

	private render() {
		let tree = h('div', {}, []);

		if (this.popupLint != null && this.popupLint < this.currentLintBoxes.length) {
			const box = this.currentLintBoxes[this.popupLint];
			tree = SuggestionBox(box, this.actions, () => {
				this.popupLint = undefined;
			});
			this.renderBox.getShadowHost().showPopover();
		} else {
			this.renderBox.getShadowHost().hidePopover();
		}

		this.renderBox.render(tree);
	}

	public updateLintBoxes(boxes: IgnorableLintBox[]) {
		this.currentLintBoxes.forEach((b) => {
			b.source.removeEventListener('pointerdown', this.pointerDownCallback as EventListener);
		});

		if (boxes.length != this.currentLintBoxes.length) {
			this.popupLint = undefined;
		}

		this.currentLintBoxes = boxes;
		this.currentLintBoxes.forEach((b) => {
			b.source.addEventListener('pointerdown', this.pointerDownCallback as EventListener);
		});

		this.render();
	}
}
