import type { UnpackedLint, UnpackedSuggestion } from './unpackLint';

export type Box = {
	/** Horizontal position in pixels */
	x: number;
	/** Vertical position in pixels */
	y: number;
	/** Width in pixels */
	width: number;
	/** Height in pixels */
	height: number;
};

export type LintBox = Box & {
	lint: UnpackedLint;
	source: HTMLElement;
	applySuggestion: (sug: UnpackedSuggestion) => void;
};

export type IgnorableLintBox = LintBox & {
	ignoreLint: () => Promise<void>;
};

/** Get a box that represents the screen. */
export function screenBox(): Box {
	return {
		x: 0,
		y: 0,
		width: window.innerWidth,
		height: window.innerHeight,
	};
}

export function isPointInBox(point: [number, number], box: Box) {
	const [x, y] = point;

	return x >= box.x && x <= box.x + box.width && y >= box.y && y <= box.y + box.height;
}

/** Check if a box would be visible on the screen if drawn. */
export function isBoxInScreen(box: Box): boolean {
	const screen = screenBox();

	// If any corner is in the screen, the box is visible.
	if (isPointInBox([box.x, box.y], screen)) {
		return true;
	}

	if (isPointInBox([box.x + box.width, box.y], screen)) {
		return true;
	}

	if (isPointInBox([box.x + box.width, box.y + box.height], screen)) {
		return true;
	}

	if (isPointInBox([box.x, box.y + box.height], screen)) {
		return true;
	}

	return false;
}
