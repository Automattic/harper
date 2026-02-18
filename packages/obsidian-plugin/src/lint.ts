import {
	combineConfig,
	type EditorState,
	type Extension,
	Facet,
	RangeSet,
	StateEffect,
	StateField,
	type Transaction,
	type TransactionSpec,
} from '@codemirror/state';
import {
	Decoration,
	type DecorationSet,
	EditorView,
	hoverTooltip,
	logException,
	showTooltip,
	type Tooltip,
	tooltips,
	ViewPlugin,
	type ViewUpdate,
	WidgetType,
} from '@codemirror/view';
import elt from 'crelt';

type Severity = 'hint' | 'info' | 'warning' | 'error';

/// Describes a problem or hint for a piece of code.
export interface Diagnostic {
	/// The start position of the relevant text.
	from: number;
	/// The end position. May be equal to `from`, though actually
	/// covering text is preferable.
	to: number;
	/// The severity of the problem. This will influence how it is
	/// displayed.
	severity: Severity;
	/// When given, add an extra CSS class to parts of the code that
	/// this diagnostic applies to.
	markClass?: string;
	/// An optional source string indicating where the diagnostic is
	/// coming from. You can put the name of your linter here, if
	/// applicable.
	source?: string;
	title?: string;
	/// The message associated with this diagnostic.
	message: string;
	/// An optional custom rendering function that displays the message
	/// as a DOM node.
	renderMessage?: (view: EditorView) => Node;
	/// An optional array of actions that can be taken on this
	/// diagnostic.
	actions?: readonly Action[];
	/// A callback for when the user selects to "ignore" the diagnostic.
	ignore?: () => void;
	/// A callback for when the user selects to "disable" the source of the diagnostic.
	disable?: () => void;
}

/// An action associated with a diagnostic.
export interface Action {
	/// The label to show to the user. Should be relatively short.
	name: string;
	/// The value to pass the title property of the button.
	title: string;
	/// Optional kind marker for keyboard command routing.
	kind?: 'suggestion' | 'dictionary';
	/// The function to call when the user activates this action. Is
	/// given the diagnostic's _current_ position, which may have
	/// changed since the creation of the diagnostic, due to editing.
	apply: (view: EditorView, from: number, to: number) => void;
}

type DiagnosticFilter = (diagnostics: readonly Diagnostic[], state: EditorState) => Diagnostic[];

interface LintConfig {
	/// Time to wait (in milliseconds) after a change before running
	/// the linter. Defaults to 750ms.
	delay?: number;
	/// Optional predicate that can be used to indicate when diagnostics
	/// need to be recomputed. Linting is always re-done on document
	/// changes.
	needsRefresh?: null | ((update: ViewUpdate) => boolean);
	/// Optional filter to determine which diagnostics produce markers
	/// in the content.
	markerFilter?: null | DiagnosticFilter;
	/// Filter applied to a set of diagnostics shown in a tooltip. No
	/// tooltip will appear if the empty set is returned.
	tooltipFilter?: null | DiagnosticFilter;
	/// Can be used to control what kind of transactions cause lint
	/// hover tooltips associated with the given document range to be
	/// hidden. By default any transaction that changes the line
	/// around the range will hide it. Returning null falls back to this
	/// behavior.
	hideOn?: (tr: Transaction, from: number, to: number) => boolean | null;
	/// When enabled (defaults to off), this will cause the lint panel
	/// to automatically open when diagnostics are found, and close when
	/// all diagnostics are resolved or removed.
	autoPanel?: boolean;
}

class SelectedDiagnostic {
	constructor(
		readonly from: number,
		readonly to: number,
		readonly diagnostic: Diagnostic,
	) {}
}

class LintState {
	constructor(
		readonly diagnostics: DecorationSet,
		readonly selected: SelectedDiagnostic | null,
		readonly commandTooltip: SelectedDiagnostic | null,
	) {}

	static init(diagnostics: readonly Diagnostic[], state: EditorState) {
		// Filter the list of diagnostics for which to create markers
		let markedDiagnostics = diagnostics;
		const diagnosticFilter = state.facet(lintConfig).markerFilter;
		if (diagnosticFilter) markedDiagnostics = diagnosticFilter(markedDiagnostics, state);

		const ranges = Decoration.set(
			markedDiagnostics.map((d: Diagnostic) => {
				// For zero-length ranges or ranges covering only a line break, create a widget
				return d.from == d.to || (d.from == d.to - 1 && state.doc.lineAt(d.from).to == d.from)
					? Decoration.widget({
							widget: new DiagnosticWidget(d),
							diagnostic: d,
						}).range(d.from)
					: Decoration.mark({
							attributes: {
								class: `cm-lintRange cm-lintRange-${d.severity}${d.markClass ? ` ${d.markClass}` : ''}`,
							},
							diagnostic: d,
						}).range(d.from, d.to);
			}),
			true,
		);
		return new LintState(ranges, null, null);
	}
}

function findDiagnostic(
	diagnostics: DecorationSet,
	diagnostic: Diagnostic | null = null,
	after = 0,
): SelectedDiagnostic | null {
	let found: SelectedDiagnostic | null = null;
	diagnostics.between(after, 1e9, (from, to, { spec }) => {
		if (diagnostic && spec.diagnostic != diagnostic) return;
		found = new SelectedDiagnostic(from, to, spec.diagnostic);
		return false;
	});
	return found;
}

interface HarperTooltipMeta {
	harperLint?: true;
	harperSource?: 'hover' | 'keyboard';
	harperDiagnostics?: readonly Diagnostic[];
	harperDiagnostic?: Diagnostic;
}

function hideTooltip(tr: Transaction, tooltip: Tooltip) {
	const from = tooltip.pos;
	const to = tooltip.end || from;
	if (tr.effects.some((e) => e.is(setCommandTooltipEffect))) return true;
	const result = tr.state.facet(lintConfig).hideOn(tr, from, to);
	if (result != null) return result;
	const line = tr.startState.doc.lineAt(tooltip.pos);
	return !!(
		tr.effects.some((e) => e.is(setDiagnosticsEffect)) ||
		tr.changes.touchesRange(line.from, Math.max(line.to, to))
	);
}

function maybeEnableLint(state: EditorState, effects: readonly StateEffect<unknown>[]) {
	return state.field(lintState, false)
		? effects
		: effects.concat(StateEffect.appendConfig.of(lintExtensions));
}

/// Returns a transaction spec which updates the current set of
/// diagnostics, and enables the lint extension if if wasn't already
/// active.
export function setDiagnostics(
	state: EditorState,
	diagnostics: readonly Diagnostic[],
): TransactionSpec {
	return {
		effects: maybeEnableLint(state, [setDiagnosticsEffect.of(diagnostics)]),
	};
}

/// The state effect that updates the set of active diagnostics. Can
/// be useful when writing an extension that needs to track these.
export const setDiagnosticsEffect = StateEffect.define<readonly Diagnostic[]>();

const movePanelSelection = StateEffect.define<SelectedDiagnostic | null>();
const setCommandTooltipEffect = StateEffect.define<SelectedDiagnostic | null>();

const lintState = StateField.define<LintState>({
	create() {
		return new LintState(Decoration.none, null, null);
	},
	update(value, tr) {
		if (tr.docChanged && value.diagnostics.size) {
			const mapped = value.diagnostics.map(tr.changes);
			value = new LintState(mapped, null, null);
		}

		for (const effect of tr.effects) {
			if (effect.is(setDiagnosticsEffect)) {
				value = LintState.init(effect.value, tr.state);
			} else if (effect.is(movePanelSelection)) {
				value = new LintState(value.diagnostics, effect.value, value.commandTooltip);
			} else if (effect.is(setCommandTooltipEffect)) {
				value = new LintState(value.diagnostics, value.selected, effect.value);
			}
		}

		return value;
	},
	provide: (f) => [EditorView.decorations.from(f, (s) => s.diagnostics)],
});

function createLintTooltip(
	state: EditorState,
	diagnostics: readonly Diagnostic[],
	from: number,
	to: number,
	source: 'hover' | 'keyboard',
): Tooltip {
	return {
		pos: from,
		end: to,
		above: state.doc.lineAt(from).to < to,
		strictSide: false,
		create(view) {
			return { dom: diagnosticsTooltip(view, diagnostics) };
		},
		harperLint: true,
		harperSource: source,
		harperDiagnostics: diagnostics,
		harperDiagnostic: diagnostics[0],
	} as Tooltip & HarperTooltipMeta;
}

function lintTooltip(view: EditorView, pos: number, side: -1 | 1) {
	const { diagnostics, commandTooltip } = view.state.field(lintState);
	let found: Diagnostic[] = [];
	let stackStart = 2e8;
	let stackEnd = 0;
	diagnostics.between(pos - (side < 0 ? 1 : 0), pos + (side > 0 ? 1 : 0), (from, to, { spec }) => {
		if (
			pos >= from &&
			pos <= to &&
			(from == to || ((pos > from || side > 0) && (pos < to || side < 0)))
		) {
			found.push(spec.diagnostic);
			stackStart = Math.min(from, stackStart);
			stackEnd = Math.max(to, stackEnd);
		}
	});

	const diagnosticFilter = view.state.facet(lintConfig).tooltipFilter;
	if (diagnosticFilter) found = diagnosticFilter(found, view.state);

	if (commandTooltip) {
		const hoveringAnotherDiagnostic = found.some((d) => d !== commandTooltip.diagnostic);
		if (hoveringAnotherDiagnostic) {
			view.dispatch({
				effects: [setCommandTooltipEffect.of(null), movePanelSelection.of(null)],
			});
		}
		return null;
	}

	if (!found.length) return null;

	return createLintTooltip(view.state, found, stackStart, stackEnd, 'hover');
}

function diagnosticsTooltip(view: EditorView, diagnostics: readonly Diagnostic[]) {
	return elt(
		'ul',
		{ class: 'cm-tooltip-lint' },
		diagnostics.map((d) => renderDiagnostic(view, d, false)),
	);
}

/// The type of a function that produces diagnostics.
export type LintSource = (
	view: EditorView,
) => readonly Diagnostic[] | Promise<readonly Diagnostic[]>;

const lintPlugin = ViewPlugin.fromClass(
	class {
		lintTime: number;
		timeout = -1;
		set = true;

		constructor(readonly view: EditorView) {
			const { delay } = view.state.facet(lintConfig);
			this.lintTime = Date.now() + delay;
			this.run = this.run.bind(this);
			this.timeout = setTimeout(this.run, delay);
		}

		run() {
			clearTimeout(this.timeout);
			const now = Date.now();
			if (now < this.lintTime - 10) {
				this.timeout = setTimeout(this.run, this.lintTime - now);
			} else {
				this.set = false;
				const { state } = this.view;
				const { sources } = state.facet(lintConfig);
				if (sources.length)
					Promise.all(sources.map((source) => Promise.resolve(source(this.view)))).then(
						(annotations) => {
							const all = annotations.reduce((a, b) => a.concat(b));
							if (this.view.state.doc == state.doc)
								this.view.dispatch(setDiagnostics(this.view.state, all));
						},
						(error) => {
							logException(this.view.state, error);
						},
					);
			}
		}

		update(update: ViewUpdate) {
			const config = update.state.facet(lintConfig);
			if (
				update.docChanged ||
				config != update.startState.facet(lintConfig) ||
				config.needsRefresh?.(update)
			) {
				this.lintTime = Date.now() + config.delay;
				if (!this.set) {
					this.set = true;
					this.timeout = setTimeout(this.run, config.delay);
				}
			}
		}

		force() {
			if (this.set) {
				this.lintTime = Date.now();
				this.run();
			}
		}

		destroy() {
			clearTimeout(this.timeout);
		}
	},
);

const lintConfig = Facet.define<
	{ source: LintSource | null; config: LintConfig },
	Required<LintConfig> & { sources: readonly LintSource[] }
>({
	combine(input) {
		return {
			sources: input.map((i) => i.source).filter((x) => x != null) as readonly LintSource[],
			...combineConfig(
				input.map((i) => i.config),
				{
					delay: 750,
					markerFilter: null,
					tooltipFilter: null,
					needsRefresh: null,
					hideOn: () => null,
				},
				{
					needsRefresh: (a, b) => (!a ? b : !b ? a : (u) => a(u) || b(u)),
				},
			),
		};
	},
});

/// Given a diagnostic source, this function returns an extension that
/// enables linting with that source. It will be called whenever the
/// editor is idle (after its content changed). If `null` is given as
/// source, this only configures the lint extension.
export function linter(source: LintSource | null, config: LintConfig = {}): Extension {
	return [lintConfig.of({ source, config }), lintPlugin, lintExtensions];
}

/// Forces any linters [configured](#lint.linter) to run when the
/// editor is idle to run right away.
export function forceLinting(view: EditorView) {
	const plugin = view.plugin(lintPlugin);
	if (plugin) plugin.force();
}

function assignKeys(actions: readonly Action[] | undefined) {
	const assigned: string[] = [];
	if (actions)
		// biome-ignore lint/suspicious/noLabelVar: reasons
		actions: for (const { name } of actions) {
			for (let i = 0; i < name.length; i++) {
				const ch = name[i];
				if (/[a-zA-Z]/.test(ch) && !assigned.some((c) => c.toLowerCase() == ch.toLowerCase())) {
					assigned.push(ch);
					continue actions;
				}
			}
			assigned.push('');
		}
	return assigned;
}

function renderDiagnostic(view: EditorView, diagnostic: Diagnostic, inPanel: boolean) {
	const keys = inPanel ? assignKeys(diagnostic.actions) : [];
	return elt(
		'li',
		{ class: `cm-diagnostic cm-diagnostic-${diagnostic.severity}` },
		elt('span', { class: 'cm-diagnosticTitle' }, diagnostic.title),
		elt(
			'span',
			{ class: 'cm-diagnosticText' },
			diagnostic.renderMessage ? diagnostic.renderMessage(view) : diagnostic.message,
		),
		elt(
			'span',
			{ class: 'cm-diagnosticActionCont' },
			diagnostic.actions?.map((action, i) => {
				let fired = false;
				const click = (e: Event) => {
					e.preventDefault();
					if (fired) return;
					fired = true;
					const found = findDiagnostic(view.state.field(lintState).diagnostics, diagnostic);
					if (found) action.apply(view, found.from, found.to);
				};
				const { name, title } = action;
				const keyIndex = keys[i] ? name.indexOf(keys[i]) : -1;
				const nameElt =
					keyIndex < 0
						? name
						: [
								name.slice(0, keyIndex),
								elt('u', name.slice(keyIndex, keyIndex + 1)),
								name.slice(keyIndex + 1),
							];
				return elt(
					'button',
					{
						type: 'button',
						class: 'cm-diagnosticAction',
						onclick: click,
						onmousedown: click,
						'aria-label': ` ${title}${keyIndex < 0 ? '' : ` (access key "${keys[i]})"`}.`,
					},
					nameElt,
				);
			}),
		),
		elt('div', { class: 'cm-diagnosticRow' }, [
			diagnostic.ignore &&
				elt(
					'div',
					{
						class: 'cm-diagnosticIgnore',
						onclick: (e) => {
							e.preventDefault();
							if (diagnostic.ignore) {
								diagnostic.ignore();
							}
						},
					},
					'Ignore Diagnostic',
				),
			diagnostic.disable &&
				elt(
					'div',
					{
						class: 'cm-diagnosticDisable',
						onclick: (e) => {
							e.preventDefault();
							if (diagnostic.disable) {
								diagnostic.disable();
							}
						},
						title: `Disable ${diagnostic.source}`,
					},
					'Disable Rule',
				),
		]),
	);
}

class DiagnosticWidget extends WidgetType {
	constructor(readonly diagnostic: Diagnostic) {
		super();
	}

	eq(other: DiagnosticWidget) {
		return other.diagnostic == this.diagnostic;
	}

	toDOM() {
		return elt('span', { class: `cm-lintPoint cm-lintPoint-${this.diagnostic.severity}` });
	}
}

function svg(content: string, attrs = `viewBox="0 0 40 40"`) {
	return `url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" ${attrs}>${encodeURIComponent(content)}</svg>')`;
}

function underline(color: string) {
	return svg(
		`<path d="m0 2.5 l2 -1.5 l1 0 l2 1.5 l1 0" stroke="${color}" fill="none" stroke-width="1"/>`,
		`width="6" height="3"`,
	);
}

const baseTheme = EditorView.baseTheme({
	'.cm-diagnostic': {
		padding: '4px',
		marginLeft: '0px',
		display: 'flex',
		flexDirection: 'column',
		whiteSpace: 'pre-wrap',
		maxHeight: 'calc(100% - var(--header-height)) !important',
	},

	'.cm-diagnosticTitle': {
		boxShadow: 'inset 0 -2px #DB2B39',
		width: 'max-content',
		fontWeight: 'bold',
	},

	'.cm-diagnosticText': {
		marginTop: '8px',
	},

	'.cm-diagnosticText p': {
		margin: '0px',
		padding: '0px',
		display: 'inline',
	},

	'.cm-diagnosticText code': {
		borderRadius: '0.25rem',
		backgroundColor: 'var(--background-secondary) !important',
		border:
			'1px solid rgb(from var(--background-secondary) calc(255 - r) calc(255 - g) calc(255 - b))',
		padding: '0.25rem',
	},

	'.cm-diagnosticActionCont': {
		display: 'flex',
		flexWrap: 'wrap',
		justifyContent: 'flex-start',
		alignItems: 'flex-start',
		alignContent: 'flex-start',
		gap: 'var(--size-4-2)',
	},

	'.cm-diagnosticRow': {
		display: 'flex',
		flexDirection: 'row',
		justifyContent: 'space-between',
	},

	'.cm-diagnosticAction': {
		font: 'inherit',
		border: 'none',
		marginTop: '8px',
		display: 'flex',
		alignItems: 'center',
		gap: 'var(--size-4-2)',
		padding: 'var(--size-4-1) var(--size-4-2)',
		cursor: 'var(--cursor)',
		fontSize: 'var(--font-ui-small)',
		borderRadius: 'var(--radius-s)',
		whiteSpace: 'nowrap',
	},

	'.cm-tooltip': {
		padding: 'var(--size-2-3) !important',
		border: '1px solid var(--background-modifier-border-hover) !important',
		backgroundColor: 'var(--background-secondary) !important',
		borderRadius: 'var(--radius-m) !important',
		boxShadow: 'var(--shadow-s) !important',
		zIndex: 'var(--layer-menu) !important',
		userSelect: 'none !important',
		overflow: 'hidden !important',
	},

	'.cm-diagnosticSource': {
		fontSize: '70%',
		opacity: 0.7,
	},

	'.cm-diagnosticIgnore': {
		padding: 'var(--size-4-1) 0px',
		fontSize: 'var(--font-ui-small)',
	},

	'.cm-diagnosticIgnore:hover': {
		textDecoration: 'underline',
	},

	'.cm-diagnosticDisable': {
		padding: 'var(--size-4-1) 0px',
		fontSize: 'var(--font-ui-small)',
	},

	'.cm-diagnosticDisable:hover': {
		textDecoration: 'underline',
	},

	'.cm-lintRange': {
		backgroundPosition: 'left bottom',
		backgroundRepeat: 'repeat-x',
		paddingBottom: '0.7px',
	},

	'.cm-lintRange-error': { backgroundImage: underline('#d11') },
	'.cm-lintRange-warning': { backgroundImage: underline('orange') },
	'.cm-lintRange-info': { backgroundImage: underline('#999') },
	'.cm-lintRange-hint': { backgroundImage: underline('#66d') },
	'.cm-lintRange-active': { backgroundColor: '#ffdd9980' },

	'.cm-tooltip-lint': {
		padding: 0,
		margin: 0,
	},

	'.cm-lintPoint': {
		position: 'relative',

		'&:after': {
			content: '""',
			position: 'absolute',
			bottom: 0,
			left: '-2px',
			borderLeft: '3px solid transparent',
			borderRight: '3px solid transparent',
			borderBottom: '4px solid #d11',
		},
	},

	'.cm-lintPoint-warning': {
		'&:after': { borderBottomColor: 'orange' },
	},
	'.cm-lintPoint-info': {
		'&:after': { borderBottomColor: '#999' },
	},
	'.cm-lintPoint-hint': {
		'&:after': { borderBottomColor: '#66d' },
	},

	'.cm-panel.cm-panel-lint': {
		position: 'relative',
		'& ul': {
			maxHeight: '100px',
			overflowY: 'auto',
			'& [aria-selected]': {
				backgroundColor: '#ddd',
				'& u': { textDecoration: 'underline' },
			},
			'&:focus [aria-selected]': {
				background_fallback: '#bdf',
				backgroundColor: 'Highlight',
				color_fallback: 'white',
				color: 'HighlightText',
			},
			'& u': { textDecoration: 'none' },
			padding: 0,
			margin: 0,
		},
		'& [name=close]': {
			position: 'absolute',
			top: '0',
			right: '2px',
			background: 'inherit',
			border: 'none',
			font: 'inherit',
			padding: 0,
			margin: 0,
		},
	},
});

const commandTooltipField = StateField.define<readonly Tooltip[]>({
	create(state) {
		const { commandTooltip } = state.field(lintState);
		return commandTooltip
			? [
					createLintTooltip(
						state,
						[commandTooltip.diagnostic],
						commandTooltip.from,
						commandTooltip.to,
						'keyboard',
					),
				]
			: [];
	},
	update(_tooltips, tr) {
		const { commandTooltip } = tr.state.field(lintState);
		return commandTooltip
			? [
					createLintTooltip(
						tr.state,
						[commandTooltip.diagnostic],
						commandTooltip.from,
						commandTooltip.to,
						'keyboard',
					),
				]
			: [];
	},
	provide: (f) => showTooltip.computeN([f], (state) => state.field(f)),
});

const lintExtensions = [
	tooltips({
		position: 'absolute',
		tooltipSpace: (view) => {
			const rect = view.dom.getBoundingClientRect();
			return {
				top: rect.top,
				left: rect.left,
				bottom: rect.bottom,
				right: rect.right,
			};
		},
	}),
	lintState,
	commandTooltipField,
	EditorView.domEventHandlers({
		mousedown(event, view) {
			const { commandTooltip, selected } = view.state.field(lintState);
			if (!commandTooltip && !selected) return false;
			const target = event.target as HTMLElement | null;
			if (target?.closest('.cm-tooltip')) return false;
			view.dispatch({
				effects: [setCommandTooltipEffect.of(null), movePanelSelection.of(null)],
			});
			return false;
		},
	}),
	hoverTooltip(lintTooltip, { hideOn: hideTooltip }),
	baseTheme,
];

/// Iterate over the marked diagnostics for the given editor state,
/// calling `f` for each of them. Note that, if the document changed
/// since the diagnostics were created, the `Diagnostic` object will
/// hold the original outdated position, whereas the `to` and `from`
/// arguments hold the diagnostic's current position.
export function forEachDiagnostic(
	state: EditorState,
	f: (d: Diagnostic, from: number, to: number) => void,
) {
	const lState = state.field(lintState, false);
	if (lState?.diagnostics.size)
		for (let iter = RangeSet.iter([lState.diagnostics]); iter.value; iter.next())
			f(iter.value.spec.diagnostic, iter.from, iter.to);
}

function collectDiagnostics(diagnostics: DecorationSet): SelectedDiagnostic[] {
	const all: SelectedDiagnostic[] = [];
	diagnostics.between(0, 1e9, (from, to, { spec }) => {
		all.push(new SelectedDiagnostic(from, to, spec.diagnostic));
	});
	return all;
}

function selectedIndexOf(all: readonly SelectedDiagnostic[], selected: SelectedDiagnostic) {
	return all.findIndex(
		(d) => d.from === selected.from && d.to === selected.to && d.diagnostic === selected.diagnostic,
	);
}

function getDiagnosticForTooltipRange(
	diagnostics: DecorationSet,
	from: number,
	to: number,
): SelectedDiagnostic | null {
	let found: SelectedDiagnostic | null = null;
	diagnostics.between(from, to, (dFrom, dTo, { spec }) => {
		if (dFrom <= to && dTo >= from) {
			found = new SelectedDiagnostic(dFrom, dTo, spec.diagnostic);
			return false;
		}
	});
	return found;
}

function getActiveTooltipMatch(view: EditorView): SelectedDiagnostic | null {
	const lState = view.state.field(lintState, false);
	if (!lState) return null;
	if (lState.commandTooltip) {
		return lState.commandTooltip;
	}

	const active = view.state.facet(showTooltip) as (Tooltip & HarperTooltipMeta)[];
	for (const tooltip of active) {
		const from = tooltip.pos;
		const to = tooltip.end ?? from;
		const matched = getDiagnosticForTooltipRange(lState.diagnostics, from, to);
		if (matched) return matched;
	}
	return null;
}

function getMappedDiagnostic(view: EditorView, diagnostic: Diagnostic): SelectedDiagnostic | null {
	const lState = view.state.field(lintState, false);
	if (!lState) return null;
	return findDiagnostic(lState.diagnostics, diagnostic);
}

function getSuggestionActions(diagnostic: Diagnostic): readonly Action[] {
	return (diagnostic.actions ?? []).filter((action) => action.kind !== 'dictionary');
}

function getDictionaryAction(diagnostic: Diagnostic): Action | null {
	for (const action of diagnostic.actions ?? []) {
		if (action.kind === 'dictionary') return action;
	}
	return null;
}

function clearKeyboardFocus(view: EditorView): boolean {
	const lState = view.state.field(lintState, false);
	if (!lState || (!lState.commandTooltip && !lState.selected)) return false;
	view.dispatch({
		selection: { anchor: view.state.selection.main.to },
		effects: [movePanelSelection.of(null), setCommandTooltipEffect.of(null)],
	});
	return true;
}

function hideVisibleLintTooltip(view: EditorView) {
	view.dispatch({
		effects: [setCommandTooltipEffect.of(null)],
	});
}

export function canNavigateDiagnostics(view: EditorView): boolean {
	const lState = view.state.field(lintState, false);
	return Boolean(lState?.diagnostics.size);
}

export function navigateDiagnostic(view: EditorView, direction: 'next' | 'previous'): boolean {
	const lState = view.state.field(lintState, false);
	if (!lState?.diagnostics.size) return false;
	const all = collectDiagnostics(lState.diagnostics);
	if (!all.length) return false;

	let target: SelectedDiagnostic | null = null;
	if (lState.selected) {
		const index = selectedIndexOf(all, lState.selected);
		if (index >= 0) {
			target =
				direction === 'next'
					? all[(index + 1) % all.length]
					: all[(index - 1 + all.length) % all.length];
		}
	}

	if (!target) {
		const cursor = view.state.selection.main.head;
		if (direction === 'next') {
			target = all.find((d) => d.from > cursor) ?? all[0];
		} else {
			target = [...all].reverse().find((d) => d.to < cursor) ?? all[all.length - 1];
		}
	}

	view.dispatch({
		selection: { anchor: target.from, head: target.to },
		effects: [movePanelSelection.of(target), setCommandTooltipEffect.of(target)],
	});
	return true;
}

export function canApplySuggestionFromVisibleTooltip(view: EditorView, index: number): boolean {
	const matched = getActiveTooltipMatch(view);
	if (!matched) return false;
	return getSuggestionActions(matched.diagnostic).length >= index;
}

export function applySuggestionFromVisibleTooltip(view: EditorView, index: number): boolean {
	const matched = getActiveTooltipMatch(view);
	if (!matched) return false;
	const action = getSuggestionActions(matched.diagnostic)[index - 1];
	if (!action) return false;
	const mapped = getMappedDiagnostic(view, matched.diagnostic) ?? matched;
	action.apply(view, mapped.from, mapped.to);
	hideVisibleLintTooltip(view);
	clearKeyboardFocus(view);
	return true;
}

export function canAddWordToDictionaryFromVisibleTooltip(view: EditorView): boolean {
	const matched = getActiveTooltipMatch(view);
	if (!matched) return false;
	return Boolean(getDictionaryAction(matched.diagnostic));
}

export function addWordToDictionaryFromVisibleTooltip(view: EditorView): boolean {
	const matched = getActiveTooltipMatch(view);
	if (!matched) return false;
	const action = getDictionaryAction(matched.diagnostic);
	if (!action) return false;
	const mapped = getMappedDiagnostic(view, matched.diagnostic) ?? matched;
	action.apply(view, mapped.from, mapped.to);
	hideVisibleLintTooltip(view);
	clearKeyboardFocus(view);
	return true;
}

export function canIgnoreVisibleTooltipDiagnostic(view: EditorView): boolean {
	const matched = getActiveTooltipMatch(view);
	if (!matched) return false;
	return typeof matched.diagnostic.ignore === 'function';
}

export function ignoreVisibleTooltipDiagnostic(view: EditorView): boolean {
	const matched = getActiveTooltipMatch(view);
	if (!matched || typeof matched.diagnostic.ignore !== 'function') return false;
	void matched.diagnostic.ignore();
	hideVisibleLintTooltip(view);
	clearKeyboardFocus(view);
	return true;
}

export function canDismissFocusedLintTooltip(view: EditorView): boolean {
	const lState = view.state.field(lintState, false);
	return Boolean(lState?.commandTooltip || lState?.selected);
}

export function dismissFocusedLintTooltip(view: EditorView): boolean {
	return clearKeyboardFocus(view);
}
