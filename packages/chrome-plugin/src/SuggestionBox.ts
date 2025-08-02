/** biome-ignore-all lint/complexity/useArrowFunction: It cannot be an arrow function for the logic to work. */
import h from 'virtual-dom/h';
import bookDownSvg from '../assets/book-down.svg?raw';
import type { IgnorableLintBox, LintBox } from './Box';
import lintKindColor from './lintKindColor';
import ProtocolClient from './ProtocolClient';
import type { UnpackedSuggestion } from './unpackLint';

var FocusHook = function () {};
FocusHook.prototype.hook = function (node, _propertyName, _previousValue) {
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

function header(title: string, color: string, onClose: () => void): any {
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

	const titleEl = h('span', {}, title);

	return h(
		'div',
		{
			className: 'harper-header',
			style: { borderBottom: `2px solid ${color}` },
		},
		[titleEl, closeButton],
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

function addToDictionary(box: LintBox): any {
	return h(
		'button',
		{
			className: 'harper-btn',
			onclick: () => {
				ProtocolClient.addToUserDictionary([box.lint.problem_text]);
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
		const label = s.replacement_text !== '' ? s.replacement_text : s.kind;
		const desc = `Replace with "${label}"`;
		const props = i === 0 ? { hook: new FocusHook() } : {};
		return button(label, { background: '#2DA44E', color: '#FFFFFF' }, () => apply(s), desc, props);
	});
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
		  font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Helvetica,Arial,sans-serif;
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
		  .harper-body{color:#8b949e}
		  .harper-btn{
		    background:#21262d;
		    color:#c9d1d9
		  }
		  .harper-btn:hover{filter:brightness(1.15)}
		  .harper-close-btn{color:#8b949e;}
		  .harper-close-btn:hover{color:#e6edf3;}
		  .harper-btn[style*="background: #2DA44E"]{background:#238636}
		  .harper-btn[style*="background: #e5e5e5"]{
		    background:#4b4b4b;
		    color:#ffffff
		  }
		}`,
	]);
}

function ignoreLint(onIgnore: () => void): any {
	return button(
		'Ignore',
		{ background: '#e5e5e5', color: '#000000', fontWeight: 'lighter' },
		onIgnore,
		'Ignore this lint',
	);
}

export default function SuggestionBox(box: IgnorableLintBox, close: () => void) {
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

	return h('div', { className: 'harper-container fade-in', style: positionStyle }, [
		styleTag(),
		header(box.lint.lint_kind_pretty, lintKindColor(box.lint.lint_kind), close),
		body(box.lint.message_html),
		footer(
			suggestions(box.lint.suggestions, (v) => {
				box.applySuggestion(v);
				close();
			}),
			[
				box.lint.lint_kind === 'Spelling' ? addToDictionary(box) : undefined,
				ignoreLint(box.ignoreLint),
			],
		),
	]);
}
