import type { IgnorableLintBox } from './Box';
import computeLintBoxes from './computeLintBoxes';
import { isVisible } from './domUtils';
import Highlights from './Highlights';
import PopupHandler from './PopupHandler';
import type { UnpackedLint } from './unpackLint';

type ActivationKey = 'off' | 'shift' | 'control';

/** Events on an input (any kind) that can trigger a re-render. */
const INPUT_EVENTS = ['focus', 'keyup', 'paste', 'change', 'scroll'];
/** Events on the window that can trigger a re-render. */
const PAGE_EVENTS = ['resize', 'scroll'];

/** Orchestrates linting and rendering in response to events on the page. */
export default class LintFramework {
	private highlights: Highlights;
	private popupHandler: PopupHandler;
	private targets: Set<Node>;
	private scrollableAncestors: Set<HTMLElement>;
	private scrollPositions: Map<HTMLElement, { left: number; top: number }>;
	private lintRequested = false;
	private renderRequested = false;
	private lastLints: { target: HTMLElement; lints: UnpackedLint[] }[] = [];
	private lastBoxes: IgnorableLintBox[] = [];

	/** The function to be called to re-render the highlights. This is a variable because it is used to register/deregister event listeners. */
	private updateEventCallback: () => void;
	/** Scroll handler used to register/deregister scroll listeners. */
	private scrollEventCallback: (ev: Event) => void;

	/** Function used to fetch lints for a given text/domain. */
	private lintProvider: (text: string, domain: string) => Promise<UnpackedLint[]>;
	/** Actions wired by host environment (extension/app). */
	private actions: {
		ignoreLint?: (hash: string) => Promise<void>;
		getActivationKey?: () => Promise<ActivationKey>;
		openOptions?: () => Promise<void>;
		addToUserDictionary?: (words: string[]) => Promise<void>;
	};

	constructor(
		lintProvider: (text: string, domain: string) => Promise<UnpackedLint[]>,
		actions: {
			ignoreLint?: (hash: string) => Promise<void>;
			getActivationKey?: () => Promise<ActivationKey>;
			openOptions?: () => Promise<void>;
			addToUserDictionary?: (words: string[]) => Promise<void>;
		},
	) {
		this.lintProvider = lintProvider;
		this.actions = actions;
		this.highlights = new Highlights();
		this.popupHandler = new PopupHandler({
			getActivationKey: actions.getActivationKey,
			openOptions: actions.openOptions,
			addToUserDictionary: actions.addToUserDictionary,
		});
		this.targets = new Set();
		this.scrollableAncestors = new Set();
		this.scrollPositions = new Map();
		this.lastLints = [];
		this.lastBoxes = [];

		this.updateEventCallback = () => {
			this.update();
		};

		this.scrollEventCallback = (ev: Event) => {
			this.onAncestorScroll(ev);
		};

		const timeoutCallback = () => {
			this.update();

			setTimeout(timeoutCallback, 100);
		};

		timeoutCallback();

		this.attachWindowListeners();
	}

	/** Returns the currents targets that are visible on-screen. */
	onScreenTargets(): Node[] {
		const onScreen = [] as Node[];

		for (const target of this.targets) {
			if (isVisible(target)) {
				onScreen.push(target);
			}
		}

		return onScreen;
	}

	async update() {
		this.requestRender();
		this.requestLintUpdate();
	}

	async requestLintUpdate() {
		if (this.lintRequested) {
			return;
		}

		// Avoid duplicate requests in the queue
		this.lintRequested = true;

		const lintResults = await Promise.all(
			this.onScreenTargets().map(async (target) => {
				if (!document.contains(target)) {
					this.targets.delete(target);
					return { target: null as HTMLElement | null, lints: [] as UnpackedLint[] };
				}

				const text =
					target instanceof HTMLTextAreaElement || target instanceof HTMLInputElement
						? target.value
						: target.textContent;

				if (!text || text.length > 120000) {
					return { target: null as HTMLElement | null, lints: [] as UnpackedLint[] };
				}

				const lints = await this.lintProvider(text, window.location.hostname);
				return { target: target as HTMLElement, lints };
			}),
		);

		this.lastLints = lintResults.filter((r) => r.target != null) as any;
		this.lintRequested = false;
		this.requestRender();
	}

	public async addTarget(target: Node) {
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

	private attachTargetListeners(target: Node) {
		for (const event of INPUT_EVENTS) {
			target.addEventListener(event, this.updateEventCallback);
		}

		const observer = new MutationObserver(this.updateEventCallback);
		const config = { subtree: true, characterData: true };

		if ((target as any).tagName == undefined) {
			observer.observe((target as any).parentElement!, config);
		} else {
			observer.observe(target as Element, config);
		}

		const scrollableAncestors = getScrollableAncestors(target);

		for (const el of scrollableAncestors) {
			if (!this.scrollableAncestors.has(el as HTMLElement)) {
				this.scrollableAncestors.add(el as HTMLElement);
				// Initialize scroll position tracking
				const scroller = el as HTMLElement;
				this.scrollPositions.set(scroller, {
					left: scroller.scrollLeft,
					top: scroller.scrollTop,
				});

				// Listen for scroll with immediate highlight shift
				(el as HTMLElement).addEventListener('scroll', this.scrollEventCallback, {
					capture: true,
					passive: true,
				});
			}
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

	/**
	 * Handle scrolls on tracked ancestor elements by shifting the last-rendered
	 * boxes immediately by the scroll delta, then schedule a full recompute.
	 */
	private onAncestorScroll(ev: Event) {
		const scroller = ev.target as HTMLElement | null;
		if (!scroller) return;

		const prev = this.scrollPositions.get(scroller);
		const current = { left: scroller.scrollLeft, top: scroller.scrollTop };
		if (!prev) {
			this.scrollPositions.set(scroller, current);
			this.updateEventCallback();
			return;
		}

		const dx = current.left - prev.left;
		const dy = current.top - prev.top;

		// Update stored position immediately
		this.scrollPositions.set(scroller, current);

		if ((dx !== 0 || dy !== 0) && this.lastBoxes.length > 0) {
			// Shift only boxes whose source is within this scroller
			const adjusted: IgnorableLintBox[] = this.lastBoxes.map((b) => {
				const sourceEl = b.source as any as HTMLElement;
				if (sourceEl && scroller.contains(sourceEl)) {
					return { ...b, x: b.x - dx, y: b.y - dy };
				}
				return b;
			});

			// Render immediately so highlights track content without visible lag
			this.highlights.renderLintBoxes(adjusted);
			this.popupHandler.updateLintBoxes(adjusted);
		}

		// Continue with normal update to recompute accurate layout
		this.updateEventCallback();
	}

	private requestRender() {
		if (this.renderRequested) {
			return;
		}

		this.renderRequested = true;

		requestAnimationFrame(() => {
			const boxes = this.lastLints.flatMap(({ target, lints }) =>
				target
					? lints.flatMap((l) =>
							computeLintBoxes(target, l as any, { ignoreLint: this.actions.ignoreLint }),
						)
					: [],
			);
			// Save for immediate scroll adjustments
			this.lastBoxes = boxes;
			this.highlights.renderLintBoxes(boxes);
			this.popupHandler.updateLintBoxes(boxes);

			this.renderRequested = false;
		});
	}
}

/**
 * Returns all scrollable ancestor elements of a given element,
 * ordered from nearest to furthest (ending with the page scroller).
 */
function getScrollableAncestors(element: Node): Element[] {
	const scrollables: Element[] = [];
	const root = document.scrollingElement || document.documentElement;
	let parent = (element as any).parentElement;

	while (parent) {
		const style = window.getComputedStyle(parent);
		const { overflowY, overflowX } = style;
		const canScrollY = overflowY.includes('auto') || overflowY.includes('scroll');
		const canScrollX = overflowX.includes('auto') || overflowX.includes('scroll');

		if (canScrollY || canScrollX) {
			scrollables.push(parent);
		}
		parent = parent.parentElement;
	}

	// Always include the document scroller at the end
	if (root && scrollables[scrollables.length - 1] !== root) {
		scrollables.push(root);
	}

	return scrollables;
}
