export type GoogleDocsRectLayout = {
	top: number;
	left: number;
	width: number;
	height: number;
};

export type GoogleDocsLineBand = {
	top: number;
	bottom: number;
};

const GOOGLE_DOCS_LINE_OVERLAP_TOLERANCE_PX = 1;
const GOOGLE_DOCS_SPACE_GAP_RATIO = 0.2;
const GOOGLE_DOCS_MIN_SPACE_GAP_PX = 3;

export function createGoogleDocsLineBand(rect: GoogleDocsRectLayout): GoogleDocsLineBand {
	return {
		top: rect.top,
		bottom: rect.top + rect.height,
	};
}

export function extendGoogleDocsLineBand(
	lineBand: GoogleDocsLineBand,
	rect: GoogleDocsRectLayout,
): GoogleDocsLineBand {
	return {
		top: Math.min(lineBand.top, rect.top),
		bottom: Math.max(lineBand.bottom, rect.top + rect.height),
	};
}

export function rectSharesGoogleDocsLineBand(
	rect: GoogleDocsRectLayout,
	lineBand: GoogleDocsLineBand,
): boolean {
	const rectBottom = rect.top + rect.height;

	return (
		rect.top <= lineBand.bottom + GOOGLE_DOCS_LINE_OVERLAP_TOLERANCE_PX &&
		rectBottom >= lineBand.top - GOOGLE_DOCS_LINE_OVERLAP_TOLERANCE_PX
	);
}

export function shouldInsertGoogleDocsSpace(
	previousRect: GoogleDocsRectLayout,
	currentRect: GoogleDocsRectLayout,
	previousText: string,
	currentText: string,
): boolean {
	if (previousText.length === 0 || currentText.length === 0) {
		return false;
	}

	if (/\s$/.test(previousText) || /^\s/.test(currentText)) {
		return false;
	}

	if (/[([{"'“‘-]$/.test(previousText) || /^[,.;:!?)\]}"'”’]/.test(currentText)) {
		return false;
	}

	const gap = currentRect.left - (previousRect.left + previousRect.width);
	const threshold = Math.max(
		GOOGLE_DOCS_MIN_SPACE_GAP_PX,
		Math.min(previousRect.height, currentRect.height) * GOOGLE_DOCS_SPACE_GAP_RATIO,
	);

	return gap >= threshold;
}
