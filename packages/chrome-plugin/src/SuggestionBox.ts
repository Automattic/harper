import h from 'virtual-dom/h';
import type { LintBox } from './Box';
import ProtocolClient from './ProtocolClient';
import lintKindColor from './lintKindColor';
import type { UnpackedLint, UnpackedSuggestion } from './unpackLint';

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

function footer(children: any) {
	const footerStyle: { [key: string]: string } = {
		display: 'flex',
		gap: '0.5rem',
		padding: '0.5rem',
		justifyContent: 'flex-end',
	};
	return h('div', { style: footerStyle }, children);
}

function addToDictionary(box: LintBox): any {
	const buttonStyle: { [key: string]: string } = {
		background: '#852387',
		color: '#ffffff',
	};

	if (box.lint.lint_kind == 'Spelling') {
		return button('Add to Dictionary', buttonStyle, () => {
			ProtocolClient.addToUserDictionary(box.lint.problem_text);
		});
	}
}

function suggestions(
	suggestions: UnpackedSuggestion[],
	apply: (sug: UnpackedSuggestion) => void,
): any {
	const suggestionButtonStyle: { [key: string]: string } = {
		background: '#238636',
		color: '#ffffff',
	};

	return suggestions.map((s: UnpackedSuggestion) => {
		const label = s.replacement_text !== '' ? s.replacement_text : s.kind;
		return button(label, suggestionButtonStyle, () => {
			apply(s);
		});
	});
}

export default function SuggestionBox(box: LintBox, close: () => void) {
	const top = box.y + box.height + 3;
	let bottom: number | undefined;
	const left = box.x;

	if (top + 400 > window.innerHeight) {
		bottom = window.innerHeight - box.y - 3;
	}

	const containerStyle: { [key: string]: string } = {
		position: 'fixed',
		top: bottom ? '' : `${top}px`,
		bottom: bottom ? `${bottom}px` : '',
		left: `${left}px`,
		maxWidth: '400px',
		maxHeight: '400px',
		background: '#ffffff',
		border: '1px solid #d0d7de',
		borderRadius: '6px',
		boxShadow: '0 2px 4px rgba(27, 31, 35, 0.15)',
		padding: '1rem',
		zIndex: '5000',
		fontFamily: 'sans-serif',
		pointerEvents: 'auto',
	};

	return h('div', { style: containerStyle }, [
		header(box.lint.lint_kind_pretty, lintKindColor(box.lint.lint_kind)),
		body(box.lint.message),
		footer(
			suggestions(box.lint.suggestions, (v) => {
				box.applySuggestion(v);
				close();
			}),
		),
		footer(addToDictionary(box)),
	]);
}
