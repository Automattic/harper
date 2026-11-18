import h from 'virtual-dom/h';
import { closestBox, type IgnorableLintBox, isPointInBox } from './Box';
import { nonEditableAncestor } from './domUtils';
import { getCaretPosition } from './editorUtils';
import type { UnpackedLint } from './unpackLint';

type ActivationKey = 'off' | 'shift' | 'control';

import hintsData from '../assets/hints.json';
import RenderBox from './RenderBox';
import SuggestionBox from './SuggestionBox';

const HOVER_ACTIVATION_DELAY_MS = 200;

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
	private selectedSuggestion = 0;
	private openSuggestionsOnHover = false;
	/** One pending dwell, identified by source, lint span, and the latest pointer position. */
	private pendingHover:
		| {
				box: IgnorableLintBox;
				point: [number, number];
				timer: ReturnType<typeof setTimeout>;
		  }
		| undefined;
	/** Only a keyboard activation moves focus into the popup; a click leaves it in the editor. */
	private autofocusSuggestion = false;
	private currentHint: string | null | undefined;
	private currentHintFor: number | undefined;
	private renderBox: RenderBox;
	private pointerDownCallback: (e: PointerEvent) => void;
	private outsidePointerDocuments: Document[] = [];
	private activationKeyListener: (() => void) | undefined;
	private readonly actions: {
		getActivationKey?: () => Promise<ActivationKey>;
		openOptions?: () => Promise<void>;
		addToUserDictionary?: (words: string[]) => Promise<void>;
		reportError?: (lint: UnpackedLint, ruleId: string) => Promise<void>;
		setRuleEnabled?: (ruleId: string, enabled: boolean) => Promise<void> | void;
	};

	constructor(actions: {
		getActivationKey?: () => Promise<ActivationKey>;
		openOptions?: () => Promise<void>;
		addToUserDictionary?: (words: string[]) => Promise<void>;
		reportError?: (lint: UnpackedLint, ruleId: string) => Promise<void>;
		setRuleEnabled?: (ruleId: string, enabled: boolean) => Promise<void> | void;
	}) {
		this.actions = actions;
		this.currentLintBoxes = [];
		this.currentHint = undefined;
		this.currentHintFor = undefined;
		this.renderBox = new RenderBox(() => nonEditableAncestor(document.body));
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

	/** Open and render the row nearest the editor caret immediately on activation. */
	private openClosestToCaret() {
		this.cancelPendingHover();
		const host = this.renderBox.getShadowHost();
		const popupFocused = document.activeElement === host;
		const caretPosition = popupFocused ? null : getCaretPosition();
		const closestIdx = caretPosition
			? closestBox(caretPosition, this.currentLintBoxes)
			: popupFocused
				? this.popupLint
				: undefined;
		if (closestIdx == null || closestIdx < 0) return;

		this.popupLint = closestIdx;
		this.selectedSuggestion = 0;
		this.autofocusSuggestion = true;
		this.render();
		if (popupFocused) this.focusSelectedSuggestion();
	}

	/** Focus the active replacement without replacing the renderer's saved editor selection. */
	private focusSelectedSuggestion() {
		const row = this.renderBox
			.getShadowHost()
			.shadowRoot?.querySelector<HTMLElement>('.harper-selected');
		row?.focus({ preventScroll: true });
		row?.scrollIntoView({ block: 'nearest' });
	}

	/** Update source listeners immediately; hover is off by default for all hosts. */
	public setOpenSuggestionsOnHover(enabled: boolean): void {
		if (this.openSuggestionsOnHover === enabled) return;
		this.openSuggestionsOnHover = enabled;
		this.cancelPendingHover();
		for (const { source } of this.currentLintBoxes) {
			if (enabled) {
				source.addEventListener('pointermove', this.onPointerMove as EventListener);
				source.addEventListener('pointerleave', this.cancelPendingHover);
			} else {
				source.removeEventListener('pointermove', this.onPointerMove as EventListener);
				source.removeEventListener('pointerleave', this.cancelPendingHover);
			}
		}
	}

	/** Cancel a pending dwell and remove its temporary interruption listeners. */
	private cancelPendingHover = (): void => {
		if (!this.pendingHover) return;
		clearTimeout(this.pendingHover.timer);
		this.pendingHover = undefined;
		document.removeEventListener('pointerdown', this.cancelPendingHover, true);
		window.removeEventListener('blur', this.cancelPendingHover);
		window.removeEventListener('scroll', this.cancelPendingHover, true);
	};

	/** Resolve the pending lint against fresh boxes instead of retaining a stale row index. */
	private pendingHoverIndex(): number {
		const pending = this.pendingHover;
		if (!pending) return -1;
		return this.currentLintBoxes.findIndex(
			(box) =>
				box.source === pending.box.source &&
				box.lint.span.start === pending.box.lint.span.start &&
				box.lint.span.end === pending.box.lint.span.end &&
				box.lint.source === pending.box.lint.source &&
				isPointInBox(pending.point, box),
		);
	}

	/** Require a 200 ms dwell on one error without moving focus or interrupting dragging. */
	private onPointerMove = (event: PointerEvent): void => {
		if (!this.openSuggestionsOnHover || event.buttons !== 0 || event.pointerType === 'touch') {
			this.cancelPendingHover();
			return;
		}
		const path = event.composedPath();
		if (path.includes(this.renderBox.getShadowHost())) {
			this.cancelPendingHover();
			return;
		}
		const point: [number, number] = [event.x, event.y];
		const index = this.currentLintBoxes.findIndex(
			(box) => path.includes(box.source) && isPointInBox(point, box),
		);
		if (this.pendingHover) this.pendingHover.point = point;
		if (index >= 0 && index !== this.popupLint && index === this.pendingHoverIndex()) return;
		this.cancelPendingHover();
		// Leaving the underline cancels activation, but does not close an open popup.
		if (index < 0 || index === this.popupLint) return;
		this.pendingHover = {
			box: this.currentLintBoxes[index],
			point,
			timer: setTimeout(() => {
				const nextIndex = this.pendingHoverIndex();
				this.cancelPendingHover();
				if (!this.openSuggestionsOnHover || nextIndex < 0) return;
				this.popupLint = nextIndex;
				this.selectedSuggestion = 0;
				this.autofocusSuggestion = false;
				this.render();
			}, HOVER_ACTIVATION_DELAY_MS),
		};
		document.addEventListener('pointerdown', this.cancelPendingHover, true);
		window.addEventListener('blur', this.cancelPendingHover);
		window.addEventListener('scroll', this.cancelPendingHover, true);
	};

	private onPointerDown(e: PointerEvent) {
		this.cancelPendingHover();
		for (let i = 0; i < this.currentLintBoxes.length; i++) {
			const box = this.currentLintBoxes[i];

			if (e.composedPath().includes(box.source) && isPointInBox([e.x, e.y], box)) {
				if (this.popupLint != null) this.close();
				this.popupLint = i;
				this.selectedSuggestion = 0;
				this.autofocusSuggestion = false;
				this.render();
				return;
			}
		}

		this.close();
	}

	private close = () => {
		this.cancelPendingHover();
		this.popupLint = undefined;
		this.selectedSuggestion = 0;
		this.autofocusSuggestion = false;
		this.render();
	};

	private onOutsidePointerDown = (event: PointerEvent) => {
		const path = event.composedPath();
		if (path.includes(this.renderBox.getShadowHost())) return;
		// Editor pointer-downs are handled by the existing source listeners.
		if (this.currentLintBoxes.some((box) => path.includes(box.source))) return;
		this.close();
	};

	/** Pointer events do not cross iframe boundaries. Keep exact documents for cleanup. */
	private syncOutsideListeners(open: boolean) {
		if (!open) {
			for (const owner of this.outsidePointerDocuments) {
				owner.removeEventListener('pointerdown', this.onOutsidePointerDown, true);
			}
			this.outsidePointerDocuments = [];
			window.removeEventListener('blur', this.close);
			window.removeEventListener('pagehide', this.close);
			return;
		}
		if (this.outsidePointerDocuments.length > 0) return;

		let owner: Window = window;
		while (true) {
			try {
				const ownerDocument = owner.document;
				ownerDocument.addEventListener('pointerdown', this.onOutsidePointerDown, true);
				this.outsidePointerDocuments.push(ownerDocument);
			} catch {
				// An inaccessible document can have an accessible ancestor.
			}
			try {
				const parent = owner.parent;
				if (parent === owner) break;
				owner = parent;
			} catch {
				// Stop if traversal fails. Frame focus loss still closes the popup.
				break;
			}
		}
		window.addEventListener('blur', this.close);
		window.addEventListener('pagehide', this.close);
	}

	private render() {
		let tree = h('div', {}, []);
		const host = this.renderBox.getShadowHost();

		this.updateHint();

		if (this.popupLint != null && this.popupLint < this.currentLintBoxes.length) {
			const box = this.currentLintBoxes[this.popupLint];

			tree = SuggestionBox({
				box,
				selectedIndex: this.selectedSuggestion,
				autofocus: this.autofocusSuggestion,
				onSelect: (index) => {
					if (index === this.selectedSuggestion) return;
					this.selectedSuggestion = index;
					this.render();
				},
				actions: this.actions,
				hint: this.currentHint ?? null,
				close: this.close,
			});
		}

		this.renderBox.render(tree);

		if (this.popupLint != null && this.popupLint < this.currentLintBoxes.length) {
			this.syncOutsideListeners(true);
			host.style.setProperty('visibility', 'visible', 'important');
			if (host.isConnected && !host.matches(':popover-open')) {
				host.showPopover();
			}
		} else {
			this.syncOutsideListeners(false);
			host.style.setProperty('visibility', 'hidden', 'important');
			if (host.isConnected && host.matches(':popover-open')) {
				host.hidePopover();
			}
		}
	}

	/** Synchronize the hint with the currently focused lint.
	 * - If no lint is open, clear the hint state.
	 * - If a different lint opens, or the hint is uninitialized, decide once (~10%).
	 */
	private updateHint() {
		if (this.popupLint == null) {
			this.currentHint = undefined;
			this.currentHintFor = undefined;
			return;
		}

		if (this.currentHintFor !== this.popupLint || this.currentHint === undefined) {
			const hints: string[] = Array.isArray(hintsData)
				? ((hintsData as unknown[]).filter((v) => typeof v === 'string') as string[])
				: [];
			const show = Math.random() < 0.1 && hints.length > 0;
			this.currentHint = show ? hints[Math.floor(Math.random() * hints.length)] : null;
			this.currentHintFor = this.popupLint;
		}
	}

	/** Preserve logical selection through geometry updates, but never retarget stale row indices. */
	public updateLintBoxes(boxes: IgnorableLintBox[]) {
		const host = this.renderBox.getShadowHost();
		const popupFocused = document.activeElement === host;
		this.currentLintBoxes.forEach((b) => {
			b.source.removeEventListener('pointerdown', this.pointerDownCallback as EventListener);
			b.source.removeEventListener('pointermove', this.onPointerMove as EventListener);
			b.source.removeEventListener('pointerleave', this.cancelPendingHover);
		});

		const previous = this.popupLint == null ? undefined : this.currentLintBoxes[this.popupLint];
		const previousSelection = previous?.lint.suggestions[this.selectedSuggestion];
		if (previous) {
			// Context hashes are location-agnostic; source and span identify the occurrence.
			// Keep the clicked fragment of a wrapped lint, rather than jumping to its first line.
			const fragment = this.currentLintBoxes
				.slice(0, this.popupLint)
				.filter(
					(box) =>
						box.source === previous.source &&
						box.lint.span.start === previous.lint.span.start &&
						box.lint.span.end === previous.lint.span.end,
				).length;
			const matches = boxes.flatMap((box, index) =>
				box.source === previous.source &&
				box.lint.span.start === previous.lint.span.start &&
				box.lint.span.end === previous.lint.span.end &&
				box.lint.source === previous.lint.source
					? [index]
					: [],
			);
			this.popupLint = matches[Math.min(fragment, matches.length - 1)];
		}
		const next = this.popupLint == null ? undefined : boxes[this.popupLint];
		this.selectedSuggestion =
			next && previousSelection
				? Math.max(
						0,
						next.lint.suggestions.findIndex(
							(suggestion) =>
								suggestion.kind === previousSelection.kind &&
								suggestion.replacement_text === previousSelection.replacement_text,
						),
					)
				: 0;
		// Index changes due to other fields or wrapped rectangles do not start a new hint session.
		if (this.popupLint != null) this.currentHintFor = this.popupLint;

		this.currentLintBoxes = boxes;
		if (this.pendingHoverIndex() < 0) this.cancelPendingHover();
		this.currentLintBoxes.forEach((b) => {
			b.source.addEventListener('pointerdown', this.pointerDownCallback as EventListener);
			if (this.openSuggestionsOnHover) {
				b.source.addEventListener('pointermove', this.onPointerMove as EventListener);
				b.source.addEventListener('pointerleave', this.cancelPendingHover);
			}
		});

		this.render();
		if (popupFocused && this.popupLint != null && !host.shadowRoot?.activeElement) {
			this.focusSelectedSuggestion();
		}
	}
}
