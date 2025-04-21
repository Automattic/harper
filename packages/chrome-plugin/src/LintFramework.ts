import { memoize } from 'lodash-es';
import Highlights from './Highlights';
import PopupHandler from './PopupHandler';
import ProtocolClient from './ProtocolClient';
import computeLintBoxes from './computeLintBoxes';

/** Events on an input (any kind) that can trigger a re-render. */
const INPUT_EVENTS = ['focus', 'keyup', 'paste', 'change', 'scroll'];
/** Events on the window that can trigger a re-render. */
const PAGE_EVENTS = ['scroll', 'resize'];

/** Orchestrates linting and rendering in response to events on the page. */
export default class LintFramework {
	private highlights: Highlights;
	private popupHandler: PopupHandler;
	private targets: Set<HTMLElement>;

	/** The function to be called to re-render the highlights. This is a variable because it is used to register/deregister event listeners. */
	private updateEventCallback: () => void;

	constructor() {
		this.highlights = new Highlights();
		this.popupHandler = new PopupHandler();
		this.targets = new Set();

		this.updateEventCallback = () => {
			this.update();
		};

		this.attachWindowListeners();
	}

	async update() {
		const boxes = [];

		for (const target of this.targets) {
			let text: string | null = null;

			if (target instanceof HTMLTextAreaElement || target instanceof HTMLInputElement) {
				text = target.value;
			} else {
				text = target.textContent;
			}

			if (text == null || text.length > 120000) {
				continue;
			}

			const lints = await ProtocolClient.lint(text, window.location.hostname);
			boxes.push(...lints.flatMap((l) => computeLintBoxes(target, l)));
		}

		this.highlights.renderLintBoxes(boxes);
		this.popupHandler.updateLintBoxes(boxes);
	}

	public async addTarget(target: HTMLElement) {
		if (!this.targets.has(target)) {
			this.targets.add(target);
			this.update();
			this.attachTargetListeners(target);
		}
	}

	public async removeTarget(target: HTMLElement) {
		if (this.targets.has(target)) {
			this.targets.delete(target);
			this.update();
			this.detachTargetListeners(target);
		} else {
			throw new Error('HTMLElement not added.');
		}
	}

	private attachTargetListeners(target: HTMLElement) {
		for (const event of INPUT_EVENTS) {
			target.addEventListener(event, this.updateEventCallback);
		}

		const observer = new MutationObserver(this.updateEventCallback);
		const config = { attributes: true, childList: true, subtree: true, characterData: true };

		console.log(target.tagName, target.parentElement);

		if (target.tagName == undefined) {
			observer.observe(target.parentElement!, config);
		} else {
			observer.observe(target, config);
		}
	}

	private detachTargetListeners(target: HTMLElement) {
		for (const event of INPUT_EVENTS) {
			target.removeEventListener(event, this.updateEventCallback);
		}
	}

	private attachWindowListeners() {
		for (const event of PAGE_EVENTS) {
			window.addEventListener(event, this.updateEventCallback);
		}
	}

	private detachWindowListeners() {
		for (const event of PAGE_EVENTS) {
			window.removeEventListener(event, this.updateEventCallback);
		}
	}
}
