import h from 'virtual-dom/h';
import lintKindColor from './lintKindColor';
import type { UnpackedLint } from './unpackLint';
import type { UnpackedSuggestion } from './unpackLint';
import { LintBox } from './Box';

function header(title: string, color: string): any {
	const headerStyle: { [key: string]: string } = {
		fontWeight: '600',
		fontSize: '1rem',
		marginBottom: '0.5rem',
		color: '#24292f',
		borderBottom: `2px solid ${color}`,
	};
	return h('div', { style: headerStyle }, title);
}

function body(message: string): any {
	const bodyStyle: { [key: string]: string } = {
		fontSize: '0.875rem',
		color: '#57606a',
		marginBottom: '1rem',
	};
	return h('div', { style: bodyStyle }, [h('p', message)]);
}

function button(
	label: string,
	extraStyle: { [key: string]: string },
	onClick: (event: Event) => void,
): any {
	const buttonStyle: { [key: string]: string } = {
		cursor: 'pointer',
		border: 'none',
		borderRadius: '4px',
		padding: '0.5rem 1rem',
		fontSize: '0.875rem',
		fontWeight: '500',
	};
	const combinedStyle = { ...buttonStyle, ...extraStyle };
	return h('button', { style: combinedStyle, onclick: onClick }, label);
}

function suggestions(suggestions: UnpackedSuggestion[], apply: (sug: UnpackedSuggestion) => void): any {
	const footerStyle: { [key: string]: string } = {
		display: 'flex',
		gap: '0.5rem',
		justifyContent: 'flex-end',
	};
	const suggestionButtonStyle: { [key: string]: string } = {
		background: '#238636',
		color: '#ffffff',
	};
	return h(
		'div',
		{ style: footerStyle },
		suggestions.map((s: UnpackedSuggestion) => {
			const label = s.replacement_text !== '' ? s.replacement_text : s.kind;
			return button(label, suggestionButtonStyle, () => {
        apply(s)
			});
		}),
	);
}

export default function SuggestionBox(box: LintBox) {
	const containerStyle: { [key: string]: string } = {
		position: 'fixed',
		top: `${box.y + box.height + 3}px`,
		left: `${box.x}px`,
		maxWidth: '400px',
		background: '#ffffff',
		border: '1px solid #d0d7de',
		borderRadius: '6px',
		boxShadow: '0 2px 4px rgba(27, 31, 35, 0.15)',
		padding: '1rem',
    zIndex: '5000'
	};

	return h('div', { style: containerStyle }, [
		header(box.lint.lint_kind_pretty, lintKindColor(box.lint.lint_kind)),
		body(box.lint.message),
		suggestions(box.lint.suggestions, box.applySuggestion),
	]);
}
