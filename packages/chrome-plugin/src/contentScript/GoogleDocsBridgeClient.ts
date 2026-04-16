const PROTOCOL_VERSION = 'harper-gdocs-bridge/v1';
const EVENT_REQUEST = 'harper:gdocs:request';
const EVENT_RESPONSE = 'harper:gdocs:response';
const EVENT_NOTIFICATION = 'harper:gdocs:notification';
const GOOGLE_DOCS_TARGET_ID = 'harper-google-docs-target';
const GOOGLE_DOCS_EVENT_SURFACE_SELECTOR = '.kix-rotatingtilemanager';
const GOOGLE_DOCS_EDITOR_SELECTOR = '.kix-appview-editor';
const GOOGLE_DOCS_TEXT_EVENT_IFRAME_SELECTOR = '.docs-texteventtarget-iframe';

type GoogleDocsRect = {
	x: number;
	y: number;
	width: number;
	height: number;
};

type GoogleDocsPoint = {
	x: number;
	y: number;
};

type GoogleDocsRequest =
	| {
			kind: 'getRects';
			start: number;
			end: number;
	  }
	| {
			kind: 'prepareReplaceText';
			start: number;
			end: number;
			replacementText: string;
			expectedText?: string;
			beforeContext?: string;
			afterContext?: string;
	  }
	| {
			kind: 'replaceText';
			start: number;
			end: number;
			replacementText: string;
			expectedText?: string;
			beforeContext?: string;
			afterContext?: string;
	  };

type GoogleDocsGetRectsResponse = {
	kind: 'getRects';
	rects: GoogleDocsRect[];
};

type GoogleDocsPrepareReplaceTextResponse = {
	kind: 'prepareReplaceText';
	ready: boolean;
	expectedNextText?: string;
	resolvedStart?: number;
	resolvedEnd?: number;
	selectedText?: string;
	selectionStart?: GoogleDocsPoint;
	selectionEnd?: GoogleDocsPoint;
	editStart?: GoogleDocsPoint;
};

type GoogleDocsResponse =
	| GoogleDocsGetRectsResponse
	| GoogleDocsPrepareReplaceTextResponse
	| {
			kind: 'replaceText';
			applied: boolean;
	  }
	| {
			kind: 'error';
			requestKind: GoogleDocsRequest['kind'];
			code: string;
			message: string;
	  };

type GoogleDocsRequestMessage = {
	protocol: string;
	requestId: string;
	request: GoogleDocsRequest;
};

type GoogleDocsResponseMessage = {
	protocol: string;
	requestId: string;
	response: GoogleDocsResponse;
};

type GoogleDocsNotificationMessage = {
	protocol: string;
	notification:
		| {
				kind: 'textUpdated';
				length: number;
		  }
		| {
				kind: 'layoutChanged';
				reason: string;
				layoutEpoch: number;
		  };
};

type PendingRequest = {
	resolve: (value: GoogleDocsResponse) => void;
	reject: (reason?: unknown) => void;
	timeoutId: number;
};

type BridgeNotificationListener = (message: GoogleDocsNotificationMessage['notification']) => void;

let googleDocsMeasurementContext: CanvasRenderingContext2D | null | undefined;

function isRecord(value: unknown): value is Record<string, unknown> {
	return value != null && typeof value === 'object';
}

function isResponseMessage(value: unknown): value is GoogleDocsResponseMessage {
	if (!isRecord(value) || value.protocol !== PROTOCOL_VERSION) {
		return false;
	}

	return (
		typeof value.requestId === 'string' &&
		isRecord(value.response) &&
		typeof value.response.kind === 'string'
	);
}

function isNotificationMessage(value: unknown): value is GoogleDocsNotificationMessage {
	if (!isRecord(value) || value.protocol !== PROTOCOL_VERSION) {
		return false;
	}

	return isRecord(value.notification) && typeof value.notification.kind === 'string';
}

function leafNodes(node: Node): Node[] {
	const children = Array.from(node.childNodes);
	if (children.length === 0) {
		return [node];
	}

	return children.flatMap((child) => leafNodes(child));
}

function getGoogleDocsMeasurementContext(): CanvasRenderingContext2D | null {
	if (googleDocsMeasurementContext !== undefined) {
		return googleDocsMeasurementContext;
	}

	const canvas = document.createElement('canvas');
	googleDocsMeasurementContext = canvas.getContext('2d');
	return googleDocsMeasurementContext;
}

function getGoogleDocsTextSliceRect(
	host: HTMLElement,
	text: string,
	startOffset: number,
	endOffset: number,
): DOMRect | null {
	const hostRect = host.getBoundingClientRect();
	if (hostRect.width <= 0 || hostRect.height <= 0) {
		return null;
	}

	const safeStart = Math.max(0, Math.min(startOffset, text.length));
	const safeEnd = Math.max(safeStart, Math.min(endOffset, text.length));
	if (safeStart === safeEnd) {
		return null;
	}

	const ctx = getGoogleDocsMeasurementContext();
	if (ctx == null) {
		return hostRect;
	}

	const style = getComputedStyle(host);
	const font =
		style.font && style.font !== ''
			? style.font
			: `${style.fontStyle} ${style.fontVariant} ${style.fontWeight} ${style.fontSize} / ${style.lineHeight} ${style.fontFamily}`;
	ctx.font = font;

	const prefix = text.slice(0, safeStart);
	const slice = text.slice(safeStart, safeEnd);
	const prefixWidth = ctx.measureText(prefix).width;
	const sliceWidth = Math.max(1, ctx.measureText(slice).width);
	const letterSpacing = Number.parseFloat(style.letterSpacing);
	const spacing = Number.isFinite(letterSpacing) ? letterSpacing : 0;
	const spacedPrefixWidth = prefixWidth + Math.max(0, safeStart - 1) * spacing;
	const spacedSliceWidth = sliceWidth + Math.max(0, safeEnd - safeStart - 1) * spacing;
	const x = hostRect.x + Math.min(hostRect.width, Math.max(0, spacedPrefixWidth));
	const width = Math.max(1, Math.min(hostRect.right - x, spacedSliceWidth));

	return new DOMRect(x, hostRect.y, width, hostRect.height);
}

function getRangeForTextSpan(target: Element, start: number, end: number): Range | null {
	const children = leafNodes(target);
	const range = document.createRange();
	let traversed = 0;
	let startFound = false;

	for (const child of children) {
		const childText = child.textContent ?? '';

		if (!startFound && traversed + childText.length > start) {
			range.setStart(child, Math.max(0, start - traversed));
			startFound = true;
		}

		if (startFound && traversed + childText.length >= end) {
			range.setEnd(child, Math.max(0, end - traversed));
			return range;
		}

		traversed += childText.length;
	}

	return null;
}

function getGoogleDocsPositionedLeafHost(child: Node, target: HTMLElement): HTMLElement | null {
	let current = child.parentElement;

	while (current != null && current !== target) {
		if (getComputedStyle(current).position === 'absolute') {
			return current;
		}

		current = current.parentElement;
	}

	return null;
}

function getGoogleDocsHighlightRects(
	target: HTMLElement,
	span: { start: number; end: number },
): DOMRect[] {
	const children = leafNodes(target);
	const rects: DOMRect[] = [];
	let traversed = 0;

	for (const child of children) {
		const childText = child.textContent ?? '';
		const childLength = childText.length;
		const childStart = traversed;
		const childEnd = childStart + childLength;
		traversed = childEnd;
		const localStart = Math.max(0, span.start - childStart);
		const localEnd = Math.min(childLength, span.end - childStart);

		if (childLength === 0 || childEnd <= span.start || childStart >= span.end) {
			continue;
		}

		const positionedHost = getGoogleDocsPositionedLeafHost(child, target);
		if (positionedHost != null) {
			const rect = getGoogleDocsTextSliceRect(positionedHost, childText, localStart, localEnd);
			if (rect != null && rect.width > 0 && rect.height > 0) {
				rects.push(rect);
				continue;
			}
		}

		const range = document.createRange();
		range.setStart(child, localStart);
		range.setEnd(child, localEnd);
		const rangeRects = Array.from(range.getClientRects()).filter(
			(rect) => rect.width > 0 && rect.height > 0,
		);
		if (rangeRects.length > 0) {
			rects.push(...rangeRects);
		}
		(range as any).detach?.();
	}

	return rects;
}

function getCommonPrefixLength(left: string, right: string): number {
	const max = Math.min(left.length, right.length);
	let length = 0;

	while (length < max && left.charCodeAt(length) === right.charCodeAt(length)) {
		length += 1;
	}

	return length;
}

function getCommonSuffixLength(left: string, right: string): number {
	const max = Math.min(left.length, right.length);
	let length = 0;

	while (
		length < max &&
		left.charCodeAt(left.length - 1 - length) === right.charCodeAt(right.length - 1 - length)
	) {
		length += 1;
	}

	return length;
}

function getLongestCommonSubsequenceLength(left: string, right: string): number {
	if (!left || !right) {
		return 0;
	}

	const previous = new Array(right.length + 1).fill(0);
	const current = new Array(right.length + 1).fill(0);

	for (let i = 1; i <= left.length; i += 1) {
		current[0] = 0;

		for (let j = 1; j <= right.length; j += 1) {
			if (left.charCodeAt(i - 1) === right.charCodeAt(j - 1)) {
				current[j] = previous[j - 1] + 1;
			} else {
				current[j] = Math.max(previous[j], current[j - 1]);
			}
		}

		for (let j = 0; j <= right.length; j += 1) {
			previous[j] = current[j];
		}
	}

	return previous[right.length];
}

function resolveReplacementRange(
	currentText: string,
	start: number,
	end: number,
	expectedText: string,
	beforeContext: string,
	afterContext: string,
) {
	const normalizedStart = Math.max(0, Math.min(start, currentText.length));
	const normalizedEnd = Math.max(normalizedStart, Math.min(end, currentText.length));
	const directText = currentText.slice(normalizedStart, normalizedEnd);

	if (!expectedText || directText === expectedText) {
		return {
			start: normalizedStart,
			end: normalizedEnd,
		};
	}

	const spanLength = normalizedEnd - normalizedStart;

	for (let delta = -12; delta <= 12; delta += 1) {
		const candidateStart = normalizedStart + delta;
		if (candidateStart < 0) {
			continue;
		}

		const candidateEnd = candidateStart + spanLength;
		if (candidateEnd > currentText.length) {
			continue;
		}

		if (currentText.slice(candidateStart, candidateEnd) === expectedText) {
			return {
				start: candidateStart,
				end: candidateEnd,
			};
		}
	}

	const beforeWindowLength = Math.max(beforeContext.length * 2, beforeContext.length + 64);
	const afterWindowLength = Math.max(afterContext.length * 2, afterContext.length + 64);
	const hits: Array<{ start: number; end: number; score: number }> = [];
	let cursor = 0;

	while (cursor <= currentText.length) {
		const index = currentText.indexOf(expectedText, cursor);
		if (index < 0) {
			break;
		}

		const indexEnd = index + expectedText.length;
		const candidateBefore = currentText.slice(Math.max(0, index - beforeWindowLength), index);
		const candidateAfter = currentText.slice(
			indexEnd,
			Math.min(currentText.length, indexEnd + afterWindowLength),
		);
		let score = 0;

		score += getLongestCommonSubsequenceLength(beforeContext, candidateBefore) * 8;
		score += getLongestCommonSubsequenceLength(afterContext, candidateAfter) * 8;
		score += getCommonPrefixLength(beforeContext, candidateBefore) * 4;
		score += getCommonSuffixLength(beforeContext, candidateBefore) * 4;
		score += getCommonPrefixLength(afterContext, candidateAfter) * 4;
		score += getCommonSuffixLength(afterContext, candidateAfter) * 4;
		score -= Math.abs(index - normalizedStart) / 1000;
		hits.push({ start: index, end: indexEnd, score });
		cursor = index + 1;
	}

	if (hits.length === 0) {
		return {
			start: normalizedStart,
			end: normalizedEnd,
		};
	}

	hits.sort((left, right) => right.score - left.score);
	return {
		start: hits[0].start,
		end: hits[0].end,
	};
}

export default class GoogleDocsBridgeClient {
	private readonly documentRef: Document;
	private readonly requestTimeoutMs: number;
	private readonly pending = new Map<string, PendingRequest>();
	private readonly notificationListeners = new Set<BridgeNotificationListener>();
	private readonly onResponseBound: EventListener;
	private readonly onNotificationBound: EventListener;

	public constructor(documentRef: Document = document, requestTimeoutMs = 5000) {
		this.documentRef = documentRef;
		this.requestTimeoutMs = requestTimeoutMs;
		this.onResponseBound = this.onResponse.bind(this);
		this.onNotificationBound = this.handleNotificationEvent.bind(this);
		this.documentRef.addEventListener(EVENT_RESPONSE, this.onResponseBound);
		this.documentRef.addEventListener(EVENT_NOTIFICATION, this.onNotificationBound);
	}

	public dispose() {
		this.documentRef.removeEventListener(EVENT_RESPONSE, this.onResponseBound);
		this.documentRef.removeEventListener(EVENT_NOTIFICATION, this.onNotificationBound);

		for (const [requestId, request] of this.pending) {
			window.clearTimeout(request.timeoutId);
			request.reject(new Error(`Google Docs bridge request "${requestId}" was disposed`));
		}

		this.pending.clear();
		this.notificationListeners.clear();
	}

	public async getRects(start: number, end: number): Promise<GoogleDocsRect[]> {
		const response = (await this.request({
			kind: 'getRects',
			start,
			end,
		})) as GoogleDocsGetRectsResponse;

		return response.rects;
	}

	public async replaceText(
		start: number,
		end: number,
		replacementText: string,
		expectedText?: string,
		beforeContext?: string,
		afterContext?: string,
	): Promise<boolean> {
		const response = await this.request({
			kind: 'replaceText',
			start,
			end,
			replacementText,
			expectedText,
			beforeContext,
			afterContext,
		});

		return response.kind === 'replaceText' && response.applied === true;
	}

	public onTextUpdated(listener: (length: number) => void): () => void {
		return this.addNotificationListener((notification) => {
			if (notification.kind === 'textUpdated') {
				listener(notification.length);
			}
		});
	}

	public onLayoutChanged(listener: (reason: string, layoutEpoch: number) => void): () => void {
		return this.addNotificationListener((notification) => {
			if (notification.kind === 'layoutChanged') {
				listener(notification.reason, notification.layoutEpoch);
			}
		});
	}

	private addNotificationListener(listener: BridgeNotificationListener): () => void {
		this.notificationListeners.add(listener);
		return () => this.notificationListeners.delete(listener);
	}

	private async request(request: GoogleDocsRequest): Promise<GoogleDocsResponse> {
		const requestId = this.createRequestId();
		const requestMessage: GoogleDocsRequestMessage = {
			protocol: PROTOCOL_VERSION,
			requestId,
			request,
		};

		return await new Promise<GoogleDocsResponse>((resolve, reject) => {
			const timeoutId = window.setTimeout(() => {
				this.pending.delete(requestId);
				reject(new Error(`Google Docs bridge request "${request.kind}" timed out`));
			}, this.requestTimeoutMs);

			this.pending.set(requestId, { resolve, reject, timeoutId });
			this.documentRef.dispatchEvent(new CustomEvent(EVENT_REQUEST, { detail: requestMessage }));
		});
	}

	private onResponse(event: Event) {
		const detail = (event as CustomEvent).detail;
		if (!isResponseMessage(detail)) {
			return;
		}

		const pendingRequest = this.pending.get(detail.requestId);
		if (!pendingRequest) {
			return;
		}

		this.pending.delete(detail.requestId);
		window.clearTimeout(pendingRequest.timeoutId);

		if (detail.response.kind === 'error') {
			pendingRequest.reject(
				new Error(detail.response.message || 'Google Docs bridge request failed'),
			);
			return;
		}

		pendingRequest.resolve(detail.response);
	}

	private handleNotificationEvent(event: Event) {
		const detail = (event as CustomEvent).detail;
		if (!isNotificationMessage(detail)) {
			return;
		}

		for (const listener of this.notificationListeners) {
			listener(detail.notification);
		}
	}

	private createRequestId(): string {
		return `gdocs-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
	}
}
