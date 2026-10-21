import type { EditorView } from '@codemirror/view';
import type { EventRef } from 'obsidian';
import type { Diagnostic } from './lint';

declare module 'obsidian' {
	interface Workspace {
		/** Harper's CodeMirror linter emits this event to update the sidebar. */
		on(
			name: 'harper:lint-updated',
			callback: (diagnostics: readonly Diagnostic[], editorView: EditorView) => void,
			ctx?: unknown,
		): EventRef;
	}
}
