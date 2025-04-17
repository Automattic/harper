import h from 'virtual-dom/h';
import { type LintBox, isPointInBox } from './Box';
import RenderBox from './RenderBox';
import SuggestionBox from './SuggestionBox';

export default class PopupHandler {
	private currentLintBoxes: LintBox[];
	private popupLint: number | undefined;
	private renderBox: RenderBox;
	private pointerDownCallback: (e: PointerEvent) => void;

	constructor() {
		this.currentLintBoxes = [];
		this.renderBox = new RenderBox();
		this.pointerDownCallback = (e) => {
			this.onPointerDown(e);
		};
	}

	private onPointerDown(e: PointerEvent) {
		console.log('pointerdown');
		console.log([e.x, e.y]);
		console.log(this.currentLintBoxes);

		for (let i = 0; i < this.currentLintBoxes.length; i++) {
			const box = this.currentLintBoxes[i];

			if (isPointInBox([e.x, e.y], box)) {
				console.log('hit');
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
			tree = SuggestionBox(box);
		}

		this.renderBox.render(tree);
	}

	public updateLintBoxes(boxes: LintBox[]) {
		this.currentLintBoxes.forEach((b) =>

			b.source.removeEventListener('pointerdown', this.pointerDownCallback),
		);
		this.currentLintBoxes = boxes;
		this.currentLintBoxes.forEach((b) =>
			b.source.addEventListener('pointerdown', this.pointerDownCallback),
		);

		if (this.popupLint ?? 0 >= boxes.length) {
			this.popupLint = undefined;
		}

		this.render();
	}
}
