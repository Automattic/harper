export type GoogleDocsRectLayout = {
  top: number;
  left: number;
  width: number;
  height: number;
};

export type GoogleDocsLineBand = {
  top: number;
  bottom: number;
  height: number;
};

const LINE_OVERLAP_RATIO = 0.45;
const MIN_LINE_OVERLAP_PX = 1;
const LINE_CENTER_DISTANCE_RATIO = 0.75;
const SPACE_GAP_RATIO = 0.2;
const MIN_SPACE_GAP_PX = 3;
const TIGHT_LEFT_PUNCTUATION = /^[,./:;!?%)\]}"'”’—–-]/u;
const TIGHT_RIGHT_PUNCTUATION = /[([{"'“‘/_—–-]$/u;

export function createGoogleDocsLineBand(
  rect: GoogleDocsRectLayout,
): GoogleDocsLineBand {
  return {
    top: rect.top,
    bottom: rect.top + rect.height,
    height: rect.height,
  };
}

export function extendGoogleDocsLineBand(
  lineBand: GoogleDocsLineBand,
  rect: GoogleDocsRectLayout,
): GoogleDocsLineBand {
  const top = Math.min(lineBand.top, rect.top);
  const bottom = Math.max(lineBand.bottom, rect.top + rect.height);

  return {
    top,
    bottom,
    height: bottom - top,
  };
}

export function rectSharesGoogleDocsLineBand(
  rect: GoogleDocsRectLayout,
  lineBand: GoogleDocsLineBand,
): boolean {
  const rectBottom = rect.top + rect.height;
  const overlap =
    Math.min(rectBottom, lineBand.bottom) - Math.max(rect.top, lineBand.top);
  const overlapThreshold = Math.max(
    MIN_LINE_OVERLAP_PX,
    Math.min(rect.height, lineBand.height) * LINE_OVERLAP_RATIO,
  );
  const rectCenter = rect.top + rect.height / 2;
  const lineCenter = lineBand.top + lineBand.height / 2;
  const lineCenterThreshold =
    Math.max(rect.height, lineBand.height) * LINE_CENTER_DISTANCE_RATIO;

  return (
    overlap >= overlapThreshold ||
    Math.abs(rectCenter - lineCenter) <= lineCenterThreshold
  );
}

export function getGoogleDocsParagraphBreakThreshold(
  lineBand: GoogleDocsLineBand,
  nextRect: GoogleDocsRectLayout,
): number {
  return Math.max(
    MIN_SPACE_GAP_PX * 2,
    Math.min(lineBand.height, nextRect.height) * 0.5,
  );
}

export function shouldInsertGoogleDocsSpace(
  previousRect: GoogleDocsRectLayout,
  currentRect: GoogleDocsRectLayout,
  previousText: string,
  currentText: string,
): boolean {
  if (!previousText || !currentText) {
    return false;
  }

  if (/\s$/u.test(previousText) || /^\s/u.test(currentText)) {
    return false;
  }

  if (
    TIGHT_RIGHT_PUNCTUATION.test(previousText) ||
    TIGHT_LEFT_PUNCTUATION.test(currentText)
  ) {
    return false;
  }

  const horizontalGap =
    currentRect.left - (previousRect.left + previousRect.width);
  const gapThreshold = Math.max(
    MIN_SPACE_GAP_PX,
    Math.min(previousRect.height, currentRect.height) * SPACE_GAP_RATIO,
  );

  return horizontalGap >= gapThreshold;
}
