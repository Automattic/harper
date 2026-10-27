const PROTOCOL_VERSION = 'harper-gdocs-bridge/v1';
const EVENT_REQUEST = 'harper:gdocs:request';
const EVENT_RESPONSE = 'harper:gdocs:response';
const EVENT_NOTIFICATION = 'harper:gdocs:notification';

type GoogleDocsRect = {
	x: number;
	y: number;
	width: number;
	height: number;
};

type GoogleDocsRequest =
	| {
			kind: 'getRects';
			start: number;
			end: number;
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

type GoogleDocsResponse =
	| GoogleDocsGetRectsResponse
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

export default class GoogleDocsBridgeClient {
	private readonly documentRef: Document;
	private readonly requestTimeoutMs: number;
	private readonly pending = new Map<string, PendingRequest>();
	private readonly notificationListeners = new Set<BridgeNotificationListener>();
	private readonly onResponseBound: EventListener;
	private readonly onNotificationBound: EventListener;

	public constructor(documentRef: Document = document, requestTimeoutMs = 2000) {
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

		return response.kind === 'replaceText' ? response.applied : false;
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
