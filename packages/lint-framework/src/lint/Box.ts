import type SourceElement from './SourceElement';
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
	source: SourceElement;
	/** Optionally provided to improve highlight rendering performance. */
	range?: Range;
	applySuggestion: (sug: UnpackedSuggestion) => void;
};

export type IgnorableLintBox = LintBox & {
	ignoreLint?: () => Promise<void>;
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

export function boxesOverlap(a: Box, b: Box): boolean {
	return a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y;
}

export function domRectToBox(rect: DOMRect): Box {
	return {
		x: rect.x,
		y: rect.y,
		width: rect.width,
		height: rect.height,
	};
}

export function isBottomEdgeInBox(inner: Box, outer: Box): boolean {
	const leftBottom: [number, number] = [inner.x, inner.y + inner.height];
	const rightBottom: [number, number] = [inner.x + inner.width, inner.y + inner.height];
	return isPointInBox(leftBottom, outer) && isPointInBox(rightBottom, outer);
}

export function closestBox(target: Box, boxes: Box[]): number {
	const cx = target.x + target.width / 2;
	const cy = target.y + target.height / 2;

	let min = Number.POSITIVE_INFINITY;
	let idx = -1;

	for (let i = 0; i < boxes.length; i++) {
		const b = boxes[i];
		if (boxesOverlap(target, b)) return i;
		const bx = b.x + b.width / 2;
		const by = b.y + b.height / 2;
		const dist = Math.hypot(bx - cx, by - cy);
		if (dist < min) {
			min = dist;
			idx = i;
		}
	}
	return idx;
}

export function shrinkBoxToFit(inner: Box, outer: Box): Box {
	const nx = Math.max(inner.x, outer.x);
	const ny = Math.max(inner.y, outer.y);
	const rx = Math.min(inner.x + inner.width, outer.x + outer.width);
	const by = Math.min(inner.y + inner.height, outer.y + outer.height);
	return { x: nx, y: ny, width: Math.max(0, rx - nx), height: Math.max(0, by - ny) };
}
