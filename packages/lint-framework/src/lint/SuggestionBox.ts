/** biome-ignore-all lint/complexity/useArrowFunction: It cannot be an arrow function for the logic to work. */
import h from 'virtual-dom/h';
import bookDownSvg from '../assets/bookDownSvg';
import type { IgnorableLintBox, LintBox } from './Box';
import lintKindColor from './lintKindColor';
// Decoupled: actions passed in by framework consumer
import type { UnpackedLint, UnpackedSuggestion } from './unpackLint';

var FocusHook: any = function () {};
FocusHook.prototype.hook = function (node: any, _propertyName: any, _previousValue: any) {
	if ((node as any).__harperAutofocused) {
		return;
	}

	requestAnimationFrame(() => {
		node.focus();
		Object.defineProperty(node, '__harperAutofocused', {
			value: true,
			enumerable: false,
			configurable: false,
		});
	});
};

/** biome-ignore-all lint/complexity/useArrowFunction: It cannot be an arrow function for the logic to work. */
var CloseOnEscapeHook: any = function (this: any, onClose: () => void) {
	this.onClose = onClose;
};

CloseOnEscapeHook.prototype.hook = function (this: { onClose: () => void }, node: HTMLElement) {
	const handler = (e: KeyboardEvent) => {
		if (e.key === 'Escape') {
			this.onClose();
		}
	};
	window.addEventListener('keydown', handler);
	(node as any).__harperCloseOnEscapeHandler = handler;
};

CloseOnEscapeHook.prototype.unhook = function (this: any, node: HTMLElement) {
	const handler = (node as any).__harperCloseOnEscapeHandler;
	if (handler) {
		window.removeEventListener('keydown', handler);
		delete (node as any).__harperCloseOnEscapeHandler;
	}
};

function header(
	title: string,
	color: string,
	onClose: () => void,
	openOptions?: () => Promise<void>,
	rule?: string,
): any {
	const closeButton = h(
		'button',
		{
			className: 'harper-close-btn',
			onclick: onClose,
			title: 'Close',
			'aria-label': 'Close',
		},
		'×',
	);

	const settingsButton = openOptions
		? h(
				'button',
				{
					className: 'harper-gear-btn',
					onclick: () => {
						openOptions();
					},
					title: 'Settings',
					'aria-label': 'Settings',
				},
				'⚙',
			)
		: undefined;

	const controlsChildren = settingsButton ? [settingsButton, closeButton] : [closeButton];
	const controls = h('div', { className: 'harper-controls' }, controlsChildren);
	const trimmedRule = rule?.trim();
	const titleChildren = [title] as any[];
	if (trimmedRule) {
		titleChildren.push(
			h(
				'span',
				{
					className: 'harper-info-icon',
					title: trimmedRule,
					'aria-label': `Grammar rule: ${trimmedRule}`,
					role: 'img',
				},
				'i',
			),
		);
	}
	const titleEl = h('span', { className: 'harper-title' }, titleChildren);

	return h(
		'div',
		{
			className: 'harper-header',
			style: { borderBottom: `2px solid ${color}` },
		},
		[titleEl, controls],
	);
}

function body(message_html: string): any {
	return h('div', { className: 'harper-body', innerHTML: message_html }, []);
}

function button(
	label: string,
	extraStyle: { [key: string]: string },
	onClick: (event: Event) => void,
	description?: string,
	extraProps: Record<string, unknown> = {},
): any {
	const desc = description || label;
	return h(
		'button',
		{
			className: 'harper-btn',
			style: extraStyle,
			onclick: onClick,
			title: desc,
			type: 'button',
			'aria-label': desc,
			...extraProps,
		},
		label,
	);
}

function footer(leftChildren: any, rightChildren: any) {
	const left = h('div', { className: 'harper-child-cont' }, leftChildren);
	const right = h('div', { className: 'harper-child-cont' }, rightChildren);
	return h('div', { className: 'harper-footer' }, [left, right]);
}

function hintDrawer(hint: string | null): any {
	if (!hint) return undefined;
	return h('div', { className: 'harper-hint-drawer', role: 'note', 'aria-live': 'polite' }, [
		h('div', { className: 'harper-hint-content' }, [
			h('div', { className: 'harper-hint-icon', 'aria-hidden': 'true' }, '💡'),
			h('div', {}, [
				h('div', { className: 'harper-hint-title' }, 'Tip'),
				h('div', {}, String(hint)),
			]),
		]),
	]);
}

function addToDictionary(
	box: LintBox,
	addToUserDictionary?: (words: string[]) => Promise<void>,
): any {
	return h(
		'button',
		{
			className: 'harper-btn',
			onclick: () => {
				addToUserDictionary?.([box.lint.problem_text]);
			},
			title: 'Add word to user dictionary',
			'aria-label': 'Add word to user dictionary',
			innerHTML: bookDownSvg,
		},
		[],
	);
}

function suggestions(
	suggestions: UnpackedSuggestion[],
	apply: (s: UnpackedSuggestion) => void,
): any {
	return suggestions.map((s: UnpackedSuggestion, i: number) => {
		const label = s.replacement_text !== '' ? s.replacement_text : String(s.kind);
		const desc = `Replace with \"${label}\"`;
		const props = i === 0 ? { hook: new FocusHook() } : {};
		return button(label, { background: '#2DA44E', color: '#FFFFFF' }, () => apply(s), desc, props);
	});
}

function reportProblemButton(reportError?: () => Promise<void>): any {
	if (!reportError) {
		return undefined;
	}

	return h(
		'button',
		{
			className: 'harper-report-link',
			type: 'button',
			onclick: () => {
				reportError();
			},
			title: 'Report an issue with this lint',
			'aria-label': 'Report an issue with this lint',
		},
		'Report',
	);
}

function styleTag() {
	return h('style', { id: 'harper-suggestion-style' }, [
		`code{
      background-color:#e3eccf;
      padding:0.125rem;
      border-radius:0.25rem
      }
      .harper-container{
      max-width:420px;
      max-height:400px;
      overflow-y:auto;
      background:#ffffff;
      border:1px solid #d0d7de;
      border-radius:8px;
      box-shadow:0 4px 12px rgba(140,149,159,0.3);
      padding:8px;
      display:flex;
      flex-direction:column;
      z-index:5000;
      font-family:-apple-system,BlinkMacSystemFont,\"Segoe UI\",Helvetica,Arial,sans-serif;
      pointer-events:auto
      }
      .harper-header{
      display:flex;
      align-items:center;
      justify-content:space-between;
      font-weight:600;
      font-size:14px;
      line-height:20px;
      color:#1f2328;
      padding-bottom:4px;
      margin-bottom:4px;
      user-select:none
      }
      .harper-title{
      display:flex;
      align-items:center;
      gap:6px;
      }
      .harper-info-icon{
      display:inline-flex;
      align-items:center;
      justify-content:center;
      width:16px;
      height:16px;
      border-radius:50%;
      background:#eaeef2;
      color:#1f2328;
      font-size:11px;
      font-weight:700;
      cursor:default;
      }
      .harper-body{
      font-size:14px;
      line-height:20px;
      color:#57606a
      }
      .harper-btn{
      display:inline-flex;
      align-items:center;
      justify-content:center;
      gap:4px;
      cursor:pointer;
      border:none;
      border-radius:6px;
      padding:3px 6px;
      min-height:28px;
      font-size:13px;
      font-weight:600;
      line-height:20px;
      transition:background 120ms ease,transform 80ms ease
      }
      .harper-btn:hover{filter:brightness(0.92)}
      .harper-btn:active{transform:scale(0.97)}
      .harper-close-btn{background:transparent;border:none;cursor:pointer;font-size:20px;line-height:1;color:#57606a;padding:0 4px;}
      .harper-close-btn:hover{color:#1f2328;}
      .harper-gear-btn{background:transparent;border:none;cursor:pointer;font-size:22px;line-height:1;color:#57606a;padding:0 4px;}
      .harper-gear-btn:hover{color:#1f2328;}
      .harper-controls{display:flex;align-items:center;gap:6px;}
      .harper-child-cont{
      display:flex;
      flex-wrap:wrap;
      justify-content:flex-end;
      gap:8px
      }
      .harper-footer{
      display:flex;
      flex-wrap:wrap;
      justify-content:space-between;
      padding:2px;
      gap:16px
      }

      /* Hint drawer styles */
      .harper-hint-drawer{
        margin-top:6px;
        border-top:1px solid #eaeef2;
        background:#f6f8fa;
        color:#3e4c59;
        border-radius:0 0 6px 6px;
      }
      .harper-hint-content{
        display:flex;
        gap:8px;
        align-items:flex-start;
        padding:8px 10px;
        font-size:13px;
        line-height:18px;
      }
      .harper-hint-icon{
        flex:0 0 auto;
        width:18px;height:18px;
        border-radius:50%;
        background:#fff3c4;
        color:#7c5e10;
        display:flex;align-items:center;justify-content:center;
        font-weight:700;
      }
      .harper-hint-title{ font-weight:600; margin-right:6px; color:#1f2328; }

    .fade-in {
      animation: fadeIn 100ms ease-in-out forwards;
    }

    @keyframes fadeIn {
      from { opacity: 0; }
      to   { opacity: 1; }
    }

      @media (prefers-color-scheme:dark){
      code{background-color:#1f2d3d;color:#c9d1d9}
      .harper-container{
      background:#0d1117;
      border-color:#30363d;
      box-shadow:0 4px 12px rgba(1,4,9,0.85)
      }
      .harper-header{color:#e6edf3}
      .harper-info-icon{background:#30363d;color:#c9d1d9}
      .harper-body{color:#8b949e}
      .harper-btn{
      background:#21262d;
      color:#c9d1d9
      }
      .harper-btn:hover{filter:brightness(1.15)}
      .harper-close-btn{color:#8b949e;}
      .harper-close-btn:hover{color:#e6edf3;}
      .harper-gear-btn{color:#8b949e;}
      .harper-gear-btn:hover{color:#e6edf3;}
      .harper-btn[style*="background: #2DA44E"]{background:#238636}
      .harper-btn[style*="background: #e5e5e5"]{
      background:#4b4b4b;
      color:#ffffff
      }
      .harper-hint-drawer{ border-top-color:#30363d; background:#151b23; color:#9aa4af; }
      .harper-hint-icon{ background:#3a2f0b; color:#f2cc60; }
      .harper-hint-title{ color:#e6edf3; }
      }
      .harper-report-link{
      margin-top:8px;
      align-self:flex-start;
      background:none;
      border:none;
      padding:0;
      color:#0969da;
      font-size:13px;
      font-weight:600;
      cursor:pointer;
      }
      .harper-report-link:hover{text-decoration:underline;}
      .harper-report-link:focus{outline:2px solid #0969da; outline-offset:2px;}
      @media (prefers-color-scheme:dark){
      .harper-report-link{color:#58a6ff;}
      }`,
	]);
}

function ignoreLint(onIgnore: () => void | Promise<void>): any {
	return button(
		'Ignore',
		{ background: '#e5e5e5', color: '#000000', fontWeight: 'lighter' },
		onIgnore,
		'Ignore this lint',
	);
}

export default function SuggestionBox(
	box: IgnorableLintBox,
	actions: {
		openOptions?: () => Promise<void>;
		addToUserDictionary?: (words: string[]) => Promise<void>;
		reportError?: (lint: UnpackedLint, ruleId: string) => Promise<void>;
	},
	hint: string | null,
	close: () => void,
) {
	const top = box.y + box.height + 3;
	let bottom: number | undefined;
	const left = box.x;

	if (top + 400 > window.innerHeight) {
		bottom = window.innerHeight - box.y - 3;
	}

	const positionStyle: { [key: string]: string } = {
		position: 'fixed',
		top: bottom ? '' : `${top}px`,
		bottom: bottom ? `${bottom}px` : '',
		left: `${left}px`,
	};

	return h(
		'div',
		{
			className: 'harper-container fade-in',
			style: positionStyle,
			'harper-close-on-escape': new CloseOnEscapeHook(close),
		},
		[
			styleTag(),
			header(
				box.lint.lint_kind_pretty,
				lintKindColor(box.lint.lint_kind),
				close,
				actions.openOptions,
				box.rule,
			),
			body(box.lint.message_html),
			footer(
				suggestions(box.lint.suggestions, (v) => {
					box.applySuggestion(v);
					close();
				}),
				[
					box.lint.lint_kind === 'Spelling' && actions.addToUserDictionary
						? addToDictionary(box, actions.addToUserDictionary)
						: undefined,
					box.ignoreLint ? ignoreLint(box.ignoreLint) : undefined,
				],
			),
			hintDrawer(hint),
			actions.reportError
				? reportProblemButton(() => {
						if (actions.reportError) {
							return actions.reportError(box.lint, box.rule);
						}
						return Promise.resolve();
					})
				: undefined,
		],
	);
}
