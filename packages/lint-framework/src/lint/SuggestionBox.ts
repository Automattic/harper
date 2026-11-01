import { icon } from '@fortawesome/fontawesome-svg-core';
import { faSliders } from '@fortawesome/free-solid-svg-icons';
import { SuggestionKind } from 'harper.js';
import { h, type VNode } from 'virtual-dom';
import bookDownSvg from '../assets/bookDownSvg';
import type { Box, IgnorableLintBox } from './Box';
import { type LintKind, lintKindColor } from './lintKindColor';
import type { UnpackedLint, UnpackedSuggestion } from './unpackLint';

const settingsIconSvg = icon(faSliders).html.join('');
const PANEL_GAP = 6;
const VIEWPORT_MARGIN = 8;

/** Saved cursor restore function, captured only when the popup first steals focus. */
let savedRestore: (() => void) | null = null;

function saveCursorState() {
	savedRestore = null;
	const el = document.activeElement;
	if (!el || el === document.body || el.tagName.toLowerCase() === 'harper-render-box') return;

	if (el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement) {
		const start = el.selectionStart;
		const end = el.selectionEnd;
		const dir = el.selectionDirection ?? undefined;
		savedRestore =
			start != null
				? () => {
						el.focus({ preventScroll: true });
						el.setSelectionRange(start, end, dir);
					}
				: () => el.focus({ preventScroll: true });
	} else if (el instanceof HTMLElement) {
		const sel = window.getSelection();
		const range = sel && sel.rangeCount > 0 ? sel.getRangeAt(0).cloneRange() : null;
		savedRestore = () => {
			el.focus({ preventScroll: true });
			if (range) {
				const s = window.getSelection();
				if (s) {
					s.removeAllRanges();
					s.addRange(range);
				}
			}
		};
	}
}

/** Defer restoration until the closing key event has finished dispatching. */
function restoreCursorState() {
	const restore = savedRestore;
	savedRestore = null;
	if (!restore) return;
	setTimeout(() => {
		try {
			restore();
		} catch {
			// A host editor may have replaced the saved selection's DOM nodes.
		}
	}, 0);
}

function clamp(value: number, min: number, max: number) {
	return Math.max(min, Math.min(value, Math.max(min, max)));
}

/**
 * Position the list at the clicked highlight and anchor the flyout to the list,
 * independently of the selected row or list scroll position.
 * Inputs and outputs are CSS pixels. Panel sizes are measured, not estimated from
 * message length. On narrow viewports, stack the panels instead of clipping them.
 */
function popupPosition(
	anchor: Box,
	viewport: { width: number; height: number },
	list: { width: number; height: number },
	flyout: { width: number; height: number },
) {
	const margin = VIEWPORT_MARGIN;
	const stacked = list.width + PANEL_GAP + flyout.width + margin * 2 > viewport.width;
	const height = stacked ? list.height + PANEL_GAP + flyout.height : list.height;
	const below = anchor.y + anchor.height + 3;
	const flipped = below + height + margin > viewport.height;
	const top = clamp(
		flipped ? anchor.y - height - 3 : below,
		margin,
		viewport.height - height - margin,
	);
	let left = clamp(anchor.x, margin, viewport.width - list.width - margin);
	const opensLeft = left + list.width + PANEL_GAP + flyout.width + margin > viewport.width;
	if (opensLeft && !stacked) {
		left = clamp(left, margin + flyout.width + PANEL_GAP, viewport.width - list.width - margin);
	}
	const flyoutLeft = stacked ? 0 : opensLeft ? -flyout.width - PANEL_GAP : list.width + PANEL_GAP;
	const flyoutTop = stacked
		? top + list.height + PANEL_GAP
		: clamp(top, margin, viewport.height - flyout.height - margin);

	return { top, left, flyoutLeft, flyoutTop: flyoutTop - top, flipped, opensLeft };
}

/**
 * virtual-dom only recognizes hooks inherited from a prototype. This functional
 * adapter pairs each DOM setup with cleanup on replacement or removal; no popup
 * selection state lives in the hook.
 */
function domHook(setup: (node: HTMLElement) => () => void) {
	let cleanup: (() => void) | undefined;
	let frame: number;
	return Object.create({
		hook(node: HTMLElement) {
			if (node.isConnected) {
				cleanup = setup(node);
			} else {
				// Creation hooks run before the node's children have been appended.
				frame = requestAnimationFrame(() => {
					if (node.isConnected) cleanup = setup(node);
				});
			}
		},
		unhook() {
			cancelAnimationFrame(frame);
			cleanup?.();
		},
	});
}

/**
 * Measure after attachment and on panel or viewport resize. Autofocus once per
 * popup DOM lifetime, not once per hover render. Clean up observers, listeners,
 * and pending animation frames when virtual-dom replaces the hook.
 */
function popupHook(props: Parameters<typeof SuggestionBox>[0], refocusClose: () => void) {
	return domHook((node) => {
		const list = node.querySelector<HTMLElement>('.harper-container')!;
		const flyout = node.querySelector<HTMLElement>('.harper-flyout')!;
		const measure = () => {
			if (!node.isConnected) return;
			const position = popupPosition(
				props.box,
				{ width: window.innerWidth, height: window.innerHeight },
				{ width: list.offsetWidth, height: list.offsetHeight },
				{ width: flyout.offsetWidth, height: flyout.offsetHeight },
			);
			node.style.top = `${position.top}px`;
			node.style.left = `${position.left}px`;
			flyout.style.left = `${position.flyoutLeft}px`;
			flyout.style.top = `${position.flyoutTop}px`;
			node.style.transformOrigin = `${position.flipped ? 'bottom' : 'top'} ${position.opensLeft ? 'right' : 'left'}`;
			node.style.visibility = 'visible';
		};
		const frame = requestAnimationFrame(() => {
			if (!node.isConnected) return;
			measure();
			if (!node.dataset.autofocused) {
				saveCursorState();
				node.dataset.autofocused = 'true';
				focusRow(node, props.selectedIndex);
			}
			measure();
		});
		const onEscape = (event: KeyboardEvent) => {
			if (event.key !== 'Escape') return;
			event.preventDefault();
			refocusClose();
		};
		const observer = new ResizeObserver(measure);
		observer.observe(list);
		observer.observe(flyout);
		window.addEventListener('resize', measure);
		window.addEventListener('keydown', onEscape);
		return () => {
			cancelAnimationFrame(frame);
			observer.disconnect();
			window.removeEventListener('resize', measure);
			window.removeEventListener('keydown', onEscape);
		};
	});
}

function focusRow(root: HTMLElement, index: number) {
	const row = root.querySelector<HTMLElement>(`#harper-suggestion-${index}`);
	row?.focus({ preventScroll: true });
	row?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
}

function suggestionKindToLabel(kind: SuggestionKind): string {
	switch (kind) {
		case SuggestionKind.Replace:
			return 'Replace';
		case SuggestionKind.Remove:
			return 'Remove';
		case SuggestionKind.InsertAfter:
			return 'Insert After';
	}
}

function suggestionLabel(suggestion: UnpackedSuggestion) {
	return suggestion.replacement_text || suggestionKindToLabel(suggestion.kind);
}

/** Describe the actual edit kind, rather than just repeating the row's visible label. */
function suggestionTitle(suggestion: UnpackedSuggestion, original: string): string {
	switch (suggestion.kind) {
		case SuggestionKind.Replace:
			return `Click to replace "${original}" with "${suggestion.replacement_text}"`;
		case SuggestionKind.Remove:
			return `Click to remove "${original}"`;
		case SuggestionKind.InsertAfter:
			return `Click to insert "${suggestion.replacement_text}" after "${original}"`;
	}
}

function button(label: string, onClick: () => void, props: Record<string, unknown> = {}) {
	return h(
		'button',
		{
			type: 'button',
			className: 'harper-btn',
			onclick: onClick,
			onmousedown: (event: MouseEvent) => event.preventDefault(),
			...props,
		},
		label,
	);
}

function hintDrawer(hint: string | null): VNode | undefined {
	if (!hint) return undefined;
	return h(
		'div',
		{ className: 'harper-hint-drawer', attributes: { role: 'note', 'aria-live': 'polite' } },
		[
			h('div', { className: 'harper-hint-content' }, [
				h('div', { className: 'harper-hint-icon', attributes: { 'aria-hidden': 'true' } }, '💡'),
				h('div', {}, [h('div', { className: 'harper-hint-title' }, 'Tip'), h('div', {}, hint)]),
			]),
		],
	);
}

/** Render one replacement, or a non-applying details row when there are no replacements. */
function suggestionRow(
	suggestion: UnpackedSuggestion | undefined,
	index: number,
	props: Parameters<typeof SuggestionBox>[0],
	apply: () => void,
) {
	const { box } = props;
	const selected = index === props.selectedIndex;
	const label = suggestion ? suggestionLabel(suggestion) : 'No replacements';
	// A stable key preserves keyboard focus when earlier alternatives are removed or reordered.
	const duplicateIndex = suggestion
		? box.lint.suggestions
				.slice(0, index)
				.filter(
					(candidate) =>
						candidate.kind === suggestion.kind &&
						candidate.replacement_text === suggestion.replacement_text,
				).length
		: 0;
	return h(
		'button',
		{
			key: JSON.stringify([suggestion?.kind, suggestion?.replacement_text, duplicateIndex]),
			id: `harper-suggestion-${index}`,
			type: 'button',
			className: `harper-row${selected ? ' harper-selected' : ''}`,
			tabIndex: selected ? 0 : -1,
			attributes: {
				role: 'menuitem',
				'aria-controls': 'harper-flyout',
				'aria-expanded': String(selected),
			},
			title: suggestion
				? suggestionTitle(suggestion, box.lint.problem_text)
				: `No replacements for "${box.lint.problem_text}"`,
			onmouseenter: () => props.onSelect(index),
			onfocus: () => props.onSelect(index),
			onclick: apply,
		},
		[
			h(
				'span',
				{
					className: 'harper-swatch',
					style: { background: lintKindColor(box.lint.lint_kind) },
					attributes: { 'aria-hidden': 'true' },
				},
				[],
			),
			h('span', { className: 'harper-row-label' }, label),
			h('span', { className: 'harper-chevron', attributes: { 'aria-hidden': 'true' } }, '›'),
		],
	);
}

/** Show the lint's explanation and secondary actions; applying belongs to the suggestion rows. */
function flyoutPanel(
	box: IgnorableLintBox,
	actions: Parameters<typeof SuggestionBox>[0]['actions'],
	runAndClose: (action: () => void | Promise<void>) => void,
) {
	const lint = box.lint;
	const color = lintKindColor(lint.lint_kind);
	const ignore = box.ignoreLint;
	const children = [
		h(
			'div',
			{ id: 'harper-kind', className: 'harper-kind', style: { borderBottomColor: color } },
			lint.lint_kind_pretty,
		),
		h('div', { className: 'harper-body', innerHTML: lint.message_html }, []),
		h('div', { className: 'harper-actions' }, [
			...(ignore
				? [
						button('Dismiss', () => runAndClose(ignore), {
							className: 'harper-btn harper-dismiss',
							title: 'Ignore this lint',
						}),
					]
				: []),
			...(lint.lint_kind === 'Spelling' && actions.addToUserDictionary
				? [
						button('', () => runAndClose(() => actions.addToUserDictionary!([lint.problem_text])), {
							title: 'Add word to user dictionary',
							attributes: { 'aria-label': 'Add word to user dictionary' },
							innerHTML: bookDownSvg,
						}),
					]
				: []),
		]),
	];
	if (actions.setRuleEnabled && box.rule) {
		children.push(
			button(
				`Disable rule ${box.rule}`,
				() => runAndClose(() => actions.setRuleEnabled!(box.rule, false)),
				{
					className: 'harper-disable',
					title: `Disable the ${box.rule} rule`,
				},
			),
		);
	}
	if (actions.reportError) {
		children.push(
			button(
				'Report',
				() => {
					void actions.reportError!(lint, box.rule);
				},
				{
					className: 'harper-report-link',
					title: 'Report an issue with this lint',
				},
			),
		);
	}
	return h(
		'div',
		{
			id: 'harper-flyout',
			className: 'harper-flyout',
			attributes: { role: 'group', 'aria-labelledby': 'harper-kind' },
		},
		children,
	);
}

function listFooter(props: Parameters<typeof SuggestionBox>[0], refocusClose: () => void) {
	const children = [];
	if (props.actions.openOptions) {
		children.push(
			h(
				'button',
				{
					type: 'button',
					className: 'harper-footer-row',
					title: 'Click to go to Harper settings',
					onclick: () => {
						void props.actions.openOptions!();
					},
				},
				[
					h(
						'span',
						{
							className: 'harper-settings-icon',
							innerHTML: settingsIconSvg,
							attributes: { 'aria-hidden': 'true' },
						},
						[],
					),
					'Go to Harper settings',
				],
			),
		);
	}
	children.push(
		button('×', refocusClose, {
			className: 'harper-close-btn',
			title: 'Close',
			attributes: { 'aria-label': 'Close' },
		}),
	);
	return h('div', { className: 'harper-list-footer' }, children);
}

function styleTag(kind: LintKind) {
	return h(
		'style',
		{ id: 'harper-suggestion-style' },
		`
.harper-popup, .harper-popup * { box-sizing:border-box; }
.harper-popup {
 position:fixed; z-index:5000; visibility:hidden; pointer-events:none;
 font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Helvetica,Arial,sans-serif;
 font-size:13px; line-height:20px; text-align:left; color:#1f2328;
}
.harper-popup button { font-family:inherit; cursor:pointer; }
.harper-container, .harper-flyout {
 background:#ffffff; border:1px solid #d0d7de; border-radius:8px;
 box-shadow:0 4px 12px rgba(140,149,159,0.3); pointer-events:auto;
 max-height:min(400px, calc(100vh - 16px)); overflow-y:auto; overscroll-behavior:contain;
}
.harper-container { position:relative; width:min(210px, calc(100vw - 16px)); padding:4px 0; }
.harper-flyout { position:absolute; width:min(190px, calc(100vw - 16px)); overflow-x:hidden; }
.harper-row {
 display:flex; align-items:center; gap:8px; height:30px; width:100%;
 padding:0 8px 0 10px; border:0; background:transparent; text-align:left;
 transition:background 120ms ease;
}
.harper-selected, .harper-footer-row:hover, .harper-disable:hover { background:rgba(0,0,0,0.045); }
.harper-swatch { width:3px; height:16px; border-radius:2px; flex:0 0 auto; }
.harper-row-label { font-size:13px; font-weight:600; color:#1f2328; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.harper-chevron { margin-left:auto; font-size:12px; color:#8b949e; }
.harper-kind { padding:6px 10px; font-size:12px; font-weight:600; border-bottom:2px solid; }
.harper-body { padding:8px 10px; font-size:12px; line-height:17px; color:#57606a; overflow-wrap:anywhere; }
.harper-body p { margin:0; }
.harper-body p + p { margin-top:8px; }
.harper-body code { text-decoration:underline solid ${lintKindColor(kind)} 2px; padding:0.125rem; border-radius:0.25rem; }
.harper-actions { display:flex; flex-wrap:wrap; gap:6px; padding:0 10px 8px; }
.harper-btn {
 display:inline-flex; align-items:center; justify-content:center; gap:4px;
 border:none; border-radius:6px; padding:3px 8px; min-height:24px;
 font-size:12px; font-weight:600; line-height:18px; overflow-wrap:anywhere; max-width:100%;
 transition:background 120ms ease,transform 80ms ease;
}
.harper-btn svg { width:18px; height:18px; }
.harper-btn:hover { filter:brightness(0.92); }
.harper-btn:active { transform:scale(0.97); }
.harper-dismiss { background:#e5e5e5; color:#000000; font-weight:400; }
.harper-disable {
 display:block; width:100%; border:0; border-top:1px solid #eaeef2;
 background:transparent; padding:6px 10px; font-size:11px; color:#8b949e; text-align:left; overflow-wrap:anywhere;
}
.harper-disable:hover { color:#57606a; }
.harper-list-footer { position:relative; border-top:1px solid #eaeef2; margin-top:4px; padding:4px 26px 0 0; min-height:32px; }
.harper-footer-row { display:flex; align-items:center; gap:6px; width:100%; border:0; background:transparent; height:28px; padding:0 10px; font-size:12px; color:#57606a; text-align:left; }
.harper-close-btn { position:absolute; right:4px; top:6px; border:0; background:transparent; color:#57606a; padding:0 4px; font-size:20px; line-height:20px; }
.harper-close-btn:hover { color:#1f2328; }
.harper-settings-icon, .harper-settings-icon svg { display:block; width:14px; height:14px; }
.harper-report-link { display:block; margin:8px 10px; background:none; border:none; padding:0; color:#0969da; font-size:13px; font-weight:600; }
.harper-report-link:hover { text-decoration:underline; }
.harper-popup button:focus-visible { outline:2px solid #0969da; outline-offset:-2px; }
.harper-hint-drawer { margin-top:6px; border-top:1px solid #eaeef2; background:#f6f8fa; color:#3e4c59; border-radius:0 0 6px 6px; }
.harper-hint-content { display:flex; gap:8px; align-items:flex-start; padding:8px 10px; font-size:13px; line-height:18px; }
.harper-hint-icon { flex:0 0 auto; width:18px; height:18px; border-radius:50%; background:#fff3c4; color:#7c5e10; display:flex; align-items:center; justify-content:center; font-weight:700; }
.harper-hint-title { font-weight:600; margin-right:6px; color:#1f2328; }
.fade-in { animation:fadeIn 100ms ease-in-out; }
@keyframes fadeIn { from { opacity:0; transform:scale(0.95); } to { opacity:1; transform:scale(1); } }
@media (max-width:421px) {
 .harper-container, .harper-flyout { max-height:min(400px, calc((100vh - 22px) / 2)); }
}
@media (prefers-reduced-motion:reduce) {
 .fade-in { animation:none; }
 .harper-row, .harper-btn { transition:none; }
 .harper-btn:active { transform:none; }
}
@media (prefers-color-scheme:dark) {
 .harper-popup, .harper-row-label, .harper-hint-title { color:#e6edf3; }
 .harper-container, .harper-flyout { background:#0d1117; border-color:#30363d; box-shadow:0 4px 12px rgba(1,4,9,0.85); }
 .harper-body, .harper-footer-row, .harper-close-btn { color:#8b949e; }
 .harper-body code { background:#1f2d3d; color:#c9d1d9; }
 .harper-selected, .harper-footer-row:hover, .harper-disable:hover { background:rgba(255,255,255,0.06); }
 .harper-disable:hover, .harper-close-btn:hover { color:#e6edf3; }
 .harper-btn { background:#21262d; color:#c9d1d9; }
 .harper-btn:hover { filter:brightness(1.15); }
 .harper-dismiss { background:#4b4b4b; color:#ffffff; }
 .harper-disable, .harper-list-footer, .harper-hint-drawer { border-top-color:#30363d; }
 .harper-hint-drawer { background:#151b23; color:#9aa4af; }
 .harper-hint-icon { background:#3a2f0b; color:#f2cc60; }
 .harper-report-link { color:#58a6ff; }
 .harper-popup button:focus-visible { outline-color:#58a6ff; }
}
`,
	);
}

/** Render a single lint's replacements and a details flyout; the caller owns selection. */
export default function SuggestionBox(props: {
	box: IgnorableLintBox;
	selectedIndex: number;
	onSelect: (index: number) => void;
	/** Optional consumer capabilities; the popup never persists configuration itself. */
	actions: {
		openOptions?: () => Promise<void>;
		addToUserDictionary?: (words: string[]) => Promise<void>;
		reportError?: (lint: UnpackedLint, ruleId: string) => Promise<void>;
		setRuleEnabled?: (ruleId: string, enabled: boolean) => Promise<void> | void;
	};
	hint: string | null;
	close: () => void;
}) {
	const { box, selectedIndex, actions, close } = props;
	const suggestions = box.lint.suggestions.length ? box.lint.suggestions : [undefined];
	const selected = suggestions[selectedIndex];
	const refocusClose = () => {
		restoreCursorState();
		close();
	};
	const runAndClose = (action: () => void | Promise<void>) => {
		void Promise.resolve().then(action).catch(console.error).finally(refocusClose);
	};
	const apply = (suggestion: UnpackedSuggestion | undefined) => {
		if (!suggestion) return;
		// Applying owns the new caret position; never restore the pre-edit selection.
		savedRestore = null;
		void Promise.resolve()
			.then(() => box.applySuggestion(suggestion))
			.catch(console.error)
			.finally(close);
	};
	const onKeyDown = (event: KeyboardEvent) => {
		const target = event.target as HTMLElement;
		const root = event.currentTarget as HTMLElement;
		const inList = target.closest('.harper-row') != null;
		if (event.key === 'ArrowLeft' && target.closest('.harper-flyout')) {
			event.preventDefault();
			event.stopPropagation();
			focusRow(root, selectedIndex);
		} else if (inList && ['ArrowDown', 'ArrowUp', 'ArrowRight', 'Enter'].includes(event.key)) {
			event.preventDefault();
			event.stopPropagation();
			if (event.key === 'Enter') apply(selected);
			else if (event.key === 'ArrowRight')
				root.querySelector<HTMLElement>('.harper-flyout button')?.focus({ preventScroll: true });
			else {
				const index =
					(selectedIndex + (event.key === 'ArrowDown' ? 1 : -1) + suggestions.length) %
					suggestions.length;
				props.onSelect(index);
				focusRow(root, index);
			}
		}
	};
	const rowNodes = suggestions.map((suggestion, index) =>
		suggestionRow(suggestion, index, props, () => apply(suggestion)),
	);
	const hint = hintDrawer(props.hint);
	return h(
		'div',
		{
			key: 'harper-suggestion-popup',
			className: 'harper-popup fade-in',
			onkeydown: onKeyDown,
			'harper-popup-hook': popupHook(props, refocusClose),
		},
		[
			styleTag(box.lint.lint_kind),
			h('div', { className: 'harper-container' }, [
				h(
					'div',
					{
						attributes: {
							role: 'menu',
							'aria-label': 'Suggestions for this lint',
							'aria-activedescendant': `harper-suggestion-${selectedIndex}`,
						},
					},
					rowNodes,
				),
				listFooter(props, refocusClose),
				...(hint ? [hint] : []),
			]),
			flyoutPanel(box, actions, runAndClose),
		],
	);
}
