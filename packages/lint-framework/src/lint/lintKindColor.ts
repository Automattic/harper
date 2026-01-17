import { getContrastingTextColor } from './utils';

// Color map will be populated from the WASM API at runtime
let LINT_KIND_COLORS: Record<string, string> = {};

/**
 * Initialize the lint kind colors from the WASM API.
 * This should be called once at startup when harper.js is available.
 */
export async function initializeLintKindColors(): Promise<void> {
    // Access the wasm module directly
    const wasm = await import('harper-wasm');
    const colorJson = wasm.get_lint_kind_colors();
    LINT_KIND_COLORS = JSON.parse(colorJson);
}

// Export the type for the lint kind keys
export type LintKind = keyof typeof LINT_KIND_COLORS;

// Export the array of all lint kind names
export const LINT_KINDS = Object.keys(LINT_KIND_COLORS) as LintKind[];

// The main function that uses the map
export function lintKindColor(lintKindKey: string): string {
	const color = LINT_KIND_COLORS[lintKindKey];
	if (!color) {
		throw new Error(`Unexpected lint kind: ${lintKindKey}. Colors not initialized. Call initializeLintKindColors() first.`);
	}
	return color;
}

export function lintKindTextColor(lintKindKeyOrColor: string): 'black' | 'white' {
	const color = LINT_KIND_COLORS[lintKindKeyOrColor] ?? lintKindKeyOrColor;
	return getContrastingTextColor(color);
}
