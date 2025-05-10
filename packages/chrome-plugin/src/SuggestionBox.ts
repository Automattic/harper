import h from 'virtual-dom/h';
import type { LintBox } from './Box';
import ProtocolClient from './ProtocolClient';
import lintKindColor from './lintKindColor';
import type { UnpackedSuggestion } from './unpackLint';

function header(title: string, color: string): any {
	const headerStyle: { [key: string]: string } = {
		display: 'flex',
		alignItems: 'center',
		justifyContent: 'space-between',
		fontWeight: '600',
		fontSize: '14px',
		lineHeight: '20px',
		color: '#1F2328',
		paddingBottom: '8px',
		marginBottom: '8px',
		borderBottom: `2px solid ${color}`,
		userSelect: 'none',
	};
	return h('div', { style: headerStyle }, title);
}

function body(message_html: string): any {
	const bodyStyle: { [key: string]: string } = {
		fontSize: '14px',
		lineHeight: '20px',
		color: '#57606A',
	};
	return h('div', { style: bodyStyle, innerHTML: message_html }, []);
}

function button(
	label: string,
	extraStyle: { [key: string]: string },
	onClick: (event: Event) => void,
): any {
	const buttonStyle: { [key: string]: string } = {
		display: 'inline-flex',
		alignItems: 'center',
		justifyContent: 'center',
		gap: '4px',
		cursor: 'pointer',
		border: 'none',
		borderRadius: '6px',
		padding: '6px 12px',
		minHeight: '28px',
		fontSize: '13px',
		fontWeight: '600',
		lineHeight: '20px',
		transition: 'background 120ms ease',
	};
	const combinedStyle = { ...buttonStyle, ...extraStyle };
	return h('button', { style: combinedStyle, onclick: onClick }, label);
}

function footer(leftChildren: any, rightChildren: any) {
	const childContStyle: { [key: string]: string } = {
		display: 'flex',
		flexWrap: 'wrap',
		justifyContent: 'flex-end',
		gap: '8px',
	};

	const left = h('div', { style: childContStyle }, leftChildren);
	const right = h('div', { style: childContStyle }, rightChildren);

	return h(
		'div',
		{
			style: {
				display: 'flex',
				flexWrap: 'wrap',
				justifyContent: 'space-between',
				padding: '4px',
				gap: '16px',
			},
		},
		[left, right],
	);
}

function addToDictionary(box: LintBox): any {
	const buttonStyle: { [key: string]: string } = {
		background: '#8250DF',
		color: '#FFFFFF',
	};
	return button('Add to Dictionary', buttonStyle, () => {
		ProtocolClient.addToUserDictionary(box.lint.problem_text);
	});
}

function suggestions(
	suggestions: UnpackedSuggestion[],
	apply: (s: UnpackedSuggestion) => void,
): any {
	const suggestionButtonStyle: { [key: string]: string } = {
		background: '#2DA44E',
		color: '#FFFFFF',
	};
	return suggestions.map((s: UnpackedSuggestion) => {
		const label = s.replacement_text !== '' ? s.replacement_text : s.kind;
		return button(label, suggestionButtonStyle, () => {
			apply(s);
		});
	});
}

function styleTag() {
	return h('style', {}, [
		`code {
			background-color: #e3eccf;
			padding: 0.25rem;
      border-radius: 0.25rem;
		}`,
	]);
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
		maxWidth: '420px',
		maxHeight: '400px',
		overflowY: 'auto',
		background: '#FFFFFF',
		border: '1px solid #D0D7DE',
		borderRadius: '8px',
		boxShadow: '0 4px 12px rgba(140,149,159,0.3)',
		padding: '16px',
		display: 'flex',
		flexDirection: 'column',
		zIndex: '5000',
		fontFamily: '-apple-system,BlinkMacSystemFont,"Segoe UI",Helvetica,Arial,sans-serif',
		pointerEvents: 'auto',
	};

	return h('div', { style: containerStyle }, [
		styleTag(),
		header(box.lint.lint_kind_pretty, lintKindColor(box.lint.lint_kind)),
		body(box.lint.message_html),
		footer(
			box.lint.lint_kind === 'Spelling' ? addToDictionary(box) : undefined,

			suggestions(box.lint.suggestions, (v) => {
				box.applySuggestion(v);
				close();
			}),
		),
	]);
}
