import h from 'virtual-dom/h';
import bookDownSvg from '../assets/book-down.svg?raw';
import type { IgnorableLintBox, LintBox } from './Box';
import ProtocolClient from './ProtocolClient';
import lintKindColor from './lintKindColor';
import type { UnpackedSuggestion } from './unpackLint';

function header(title: string, color: string): any {
	return h(
		'div',
		{
			className: 'harper-header',
			style: { borderBottom: `2px solid ${color}` },
		},
		title,
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
	console.log(bookDownSvg);
	return h(
		'button',
		{
			className: 'harper-btn',
			onclick: () => {
				ProtocolClient.addToUserDictionary(box.lint.problem_text);
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
	return suggestions.map((s: UnpackedSuggestion) => {
		const label = s.replacement_text !== '' ? s.replacement_text : s.kind;
		const desc = `Replace with \"${label}\"`;
		return button(label, { background: '#2DA44E', color: '#FFFFFF' }, () => apply(s), desc);
	});
}

function styleTag() {
	return h('style', { id: 'harper-suggestion-style' }, [
		`code {
      background-color: #e3eccf;
      padding: 0.125rem;
      border-radius: 0.25rem;
    }

    .harper-container {
      max-width: 420px;
      max-height: 400px;
      overflow-y: auto;
      background: #ffffff;
      border: 1px solid #d0d7de;
      border-radius: 8px;
      box-shadow: 0 4px 12px rgba(140, 149, 159, 0.3);
      padding: 8px;
      display: flex;
      flex-direction: column;
      z-index: 5000;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
      pointer-events: auto;
    }

    .harper-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-weight: 600;
      font-size: 14px;
      line-height: 20px;
      color: #1f2328;
      padding-bottom: 4px;
      margin-bottom: 4px;
      user-select: none;
    }

    .harper-body {
      font-size: 14px;
      line-height: 20px;
      color: #57606a;
    }

    .harper-btn {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 4px;
      cursor: pointer;
      border: none;
      border-radius: 6px;
      padding: 3px 6px;
      min-height: 28px;
      font-size: 13px;
      font-weight: 600;
      line-height: 20px;
      transition: background 120ms ease, transform 80ms ease;
    }

    .harper-btn:hover {
      filter: brightness(0.92);
    }

    .harper-btn:active {
      transform: scale(0.97);
    }

    .harper-child-cont {
      display: flex;
      flex-wrap: wrap;
      justify-content: flex-end;
      gap: 8px;
    }

    .harper-footer {
      display: flex;
      flex-wrap: wrap;
      justify-content: space-between;
      padding: 2px;
      gap: 16px;
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

	return h('div', { className: 'harper-container', style: positionStyle }, [
		styleTag(),
		header(box.lint.lint_kind_pretty, lintKindColor(box.lint.lint_kind)),
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
