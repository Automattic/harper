/**
 * LSP type-import marker for shared protocol typedefs.
 * @typedef {import('./google-docs-protocol.js').GoogleDocsGetRectsRequest} GoogleDocsGetRectsRequest
 * @typedef {import('./google-docs-protocol.js').GoogleDocsReplaceTextRequest} GoogleDocsReplaceTextRequest
 * @typedef {import('./google-docs-protocol.js').GoogleDocsRequest} GoogleDocsRequest
 * @typedef {import('./google-docs-protocol.js').GoogleDocsRequestMessage} GoogleDocsRequestMessage
 * @typedef {import('./google-docs-protocol.js').GoogleDocsResponse} GoogleDocsResponse
 * @typedef {import('./google-docs-protocol.js').GoogleDocsResponseMessage} GoogleDocsResponseMessage
 */

const PROTOCOL_VERSION = 'harper-gdocs-bridge/v1';
const EVENT_REQUEST = 'harper:gdocs:request';
const EVENT_RESPONSE = 'harper:gdocs:response';

/**
 * @callback GoogleDocsGetRectsRequestHandler
 * @param {GoogleDocsGetRectsRequest} request
 * @returns {Promise<GoogleDocsResponse>|GoogleDocsResponse}
 */

/**
 * @callback GoogleDocsReplaceTextRequestHandler
 * @param {GoogleDocsReplaceTextRequest} request
 * @returns {Promise<GoogleDocsResponse>|GoogleDocsResponse}
 */

/**
 * Request dispatcher used by the Google Docs bridge (main-world script).
 * It mirrors the background-script style of request handling.
 */
export class GoogleDocsBridgeRequestHandler {
	/**
	 * @param {{
	 *   onGetRectsRequest: GoogleDocsGetRectsRequestHandler,
	 *   onReplaceTextRequest: GoogleDocsReplaceTextRequestHandler
	 * }} handlers
	 */
	constructor(handlers) {
		this.handlers = handlers;
		this.onRequestEventBound = this.onRequestEvent.bind(this);
	}

	start() {
		document.addEventListener(EVENT_REQUEST, this.onRequestEventBound);
	}

	stop() {
		document.removeEventListener(EVENT_REQUEST, this.onRequestEventBound);
	}

	/**
	 * @param {Event} event
	 */
	async onRequestEvent(event) {
		const detail = /** @type {CustomEvent} */ (event).detail;
		if (!this.isRequestMessage(detail)) {
			return;
		}

		const requestMessage = /** @type {GoogleDocsRequestMessage} */ (detail);
		const { request } = requestMessage;
		try {
			if (request.kind === 'getRects') {
				const response = await this.handlers.onGetRectsRequest(request);
				this.sendResponse(requestMessage.requestId, response);
				return;
			}

			if (request.kind === 'replaceText') {
				const response = await this.handlers.onReplaceTextRequest(request);
				this.sendResponse(requestMessage.requestId, response);
				return;
			}

			this.sendErrorResponse(requestMessage.requestId, request.kind, 'unsupported_request', 'Unsupported request kind');
		} catch (err) {
			this.sendErrorResponse(
				requestMessage.requestId,
				request.kind,
				'handler_error',
				err instanceof Error ? err.message : 'Unknown bridge handler error',
			);
		}
	}

	/**
	 * @param {string} requestId
	 * @param {GoogleDocsResponse} response
	 */
	sendResponse(requestId, response) {
		/** @type {GoogleDocsResponseMessage} */
		const message = {
			protocol: PROTOCOL_VERSION,
			requestId,
			response,
		};
		document.dispatchEvent(new CustomEvent(EVENT_RESPONSE, { detail: message }));
	}

	/**
	 * @param {string} requestId
	 * @param {GoogleDocsRequest['kind']} requestKind
	 * @param {string} code
	 * @param {string} message
	 */
	sendErrorResponse(requestId, requestKind, code, message) {
		this.sendResponse(requestId, {
			kind: 'error',
			requestKind,
			code,
			message,
		});
	}

	/**
	 * @param {unknown} value
	 * @returns {value is GoogleDocsRequestMessage}
	 */
	isRequestMessage(value) {
		if (!this.isObject(value)) return false;
		return (
			value.protocol === PROTOCOL_VERSION &&
			typeof value.requestId === 'string' &&
			this.isObject(value.request) &&
			typeof value.request.kind === 'string'
		);
	}

	/**
	 * @param {unknown} value
	 * @returns {value is Record<string, unknown>}
	 */
	isObject(value) {
		return value != null && typeof value === 'object';
	}
}
