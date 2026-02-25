// @ts-nocheck

import type {
	GoogleDocsGetRectsResponse,
	GoogleDocsNotificationMessage,
	GoogleDocsRequest,
	GoogleDocsRequestMessage,
	GoogleDocsResponse,
	GoogleDocsResponseMessage,
	GoogleDocsRect,
} from '../../public/google-docs-protocol.js';

type PendingRequest = {
	resolve: (value: GoogleDocsResponse) => void;
	reject: (reason?: unknown) => void;
	timerId: number;
};

const PROTOCOL_VERSION = 'harper-gdocs-bridge/v1';
const EVENT_REQUEST = 'harper:gdocs:request';
const EVENT_RESPONSE = 'harper:gdocs:response';
const EVENT_NOTIFICATION = 'harper:gdocs:notification';

/** Content-script client for Google Docs bridge request/response + notifications. */
export default class GoogleDocsBridgeClient {
	private readonly documentRef: Document;
	private readonly timeoutMs: number;
	private readonly pending = new Map<string, PendingRequest>();
	private readonly onResponseBound: EventListener;
	private readonly onNotificationBound: EventListener;

	private readonly textUpdatedListeners = new Set<(length: number) => void>();
	private readonly layoutChangedListeners = new Set<(reason: string, layoutEpoch: number) => void>();

	public constructor(documentRef: Document = document, timeoutMs = 2000) {
		this.documentRef = documentRef;
		this.timeoutMs = timeoutMs;
		this.onResponseBound = this.onResponse.bind(this);
		this.onNotificationBound = this.onNotification.bind(this);
		this.documentRef.addEventListener(EVENT_RESPONSE, this.onResponseBound);
		this.documentRef.addEventListener(EVENT_NOTIFICATION, this.onNotificationBound);
	}

	/** Remove listeners and reject any pending bridge requests. */
	public dispose() {
		this.documentRef.removeEventListener(EVENT_RESPONSE, this.onResponseBound);
		this.documentRef.removeEventListener(EVENT_NOTIFICATION, this.onNotificationBound);
		for (const [requestId, pending] of this.pending.entries()) {
			clearTimeout(pending.timerId);
			pending.reject(new Error(`Google Docs bridge request "${requestId}" was disposed`));
		}
		this.pending.clear();
		this.textUpdatedListeners.clear();
		this.layoutChangedListeners.clear();
	}

	/** Get on-screen rects for a text span in the Google Doc. */
	public async getRects(start: number, end: number): Promise<GoogleDocsRect[]> {
		const response = (await this.request({ kind: 'getRects', start, end })) as GoogleDocsGetRectsResponse;
		return response.rects;
	}

	/** Replace a text span in the Google Doc. Returns true if the edit was applied. */
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
		return response.kind === 'replaceText' ? response.applied : false;
	}

	/** Listen for bridge text updates. Returns an unsubscribe function. */
	public onTextUpdated(cb: (length: number) => void): () => void {
		this.textUpdatedListeners.add(cb);
		return () => this.textUpdatedListeners.delete(cb);
	}

	/** Listen for bridge layout changes. Returns an unsubscribe function. */
	public onLayoutChanged(cb: (reason: string, layoutEpoch: number) => void): () => void {
		this.layoutChangedListeners.add(cb);
		return () => this.layoutChangedListeners.delete(cb);
	}

	/** Send one bridge request and wait for the response with the same request id. */
	private async request(request: GoogleDocsRequest): Promise<GoogleDocsResponse> {
		const requestId = this.createRequestId();
		const message: GoogleDocsRequestMessage = {
			protocol: PROTOCOL_VERSION,
			requestId,
			request,
		};

		return new Promise((resolve, reject) => {
			const timerId = window.setTimeout(() => {
				this.pending.delete(requestId);
				reject(new Error(`Google Docs bridge request "${request.kind}" timed out`));
			}, this.timeoutMs);

			this.pending.set(requestId, { resolve, reject, timerId });
			this.documentRef.dispatchEvent(new CustomEvent(EVENT_REQUEST, { detail: message }));
		});
	}

	/** Route bridge responses to the matching pending request. */
	private onResponse(event: Event) {
		const detail = (event as CustomEvent).detail;
		if (!this.isResponseMessage(detail)) {
			return;
		}

		const pending = this.pending.get(detail.requestId);
		if (!pending) {
			return;
		}

		this.pending.delete(detail.requestId);
		clearTimeout(pending.timerId);
		if (detail.response.kind === 'error') {
			pending.reject(new Error(detail.response.message || 'Google Docs bridge request failed'));
			return;
		}

		pending.resolve(detail.response);
	}

	/** Route bridge notifications to local subscribers. */
	private onNotification(event: Event) {
		const detail = (event as CustomEvent).detail;
		if (!this.isNotificationMessage(detail)) {
			return;
		}

		const { notification } = detail;
		if (notification.kind === 'textUpdated') {
			for (const listener of this.textUpdatedListeners) {
				listener(notification.length);
			}
			return;
		}

		if (notification.kind === 'layoutChanged') {
			for (const listener of this.layoutChangedListeners) {
				listener(notification.reason, notification.layoutEpoch);
			}
		}
	}

	/** Create a request id for request/response matching. */
	private createRequestId(): string {
		return `gdocs-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
	}

	private isResponseMessage(value: unknown): value is GoogleDocsResponseMessage {
		if (!this.isObject(value)) return false;
		return (
			value.protocol === PROTOCOL_VERSION &&
			typeof value.requestId === 'string' &&
			this.isObject(value.response) &&
			typeof value.response.kind === 'string'
		);
	}

	private isNotificationMessage(value: unknown): value is GoogleDocsNotificationMessage {
		if (!this.isObject(value)) return false;
		return (
			value.protocol === PROTOCOL_VERSION &&
			this.isObject(value.notification) &&
			typeof value.notification.kind === 'string'
		);
	}

	private isObject(value: unknown): value is Record<string, any> {
		return value != null && typeof value === 'object';
	}
}
