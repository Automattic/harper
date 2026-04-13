/**
 * Type-only protocol specification for Google Docs bridge communication.
 *
 * This mirrors `protocol.ts` style:
 * - A single list of request and response types.
 * - No runtime code or constants.
 * - Transport details are handled by client/handler implementations.
 */

/**
 * A rectangle in viewport coordinates.
 * @typedef {object} GoogleDocsRect
 * @property {number} x Left coordinate in CSS pixels.
 * @property {number} y Top coordinate in CSS pixels.
 * @property {number} width Width in CSS pixels.
 * @property {number} height Height in CSS pixels.
 */

/**
 * Request to resolve rects for a text span.
 * @typedef {object} GoogleDocsGetRectsRequest
 * @property {'getRects'} kind Request kind.
 * @property {number} start Start offset in bridge text.
 * @property {number} end End offset in bridge text.
 */

/**
 * Request to resolve and select the live Google Docs range for a replacement.
 * @typedef {object} GoogleDocsPrepareReplaceTextRequest
 * @property {'prepareReplaceText'} kind Request kind.
 * @property {number} start Start offset in bridge text.
 * @property {number} end End offset in bridge text.
 * @property {string} replacementText Replacement text.
 * @property {string=} expectedText Optional expected source text for defensive re-alignment.
 * @property {string=} beforeContext Optional context before the span for re-alignment scoring.
 * @property {string=} afterContext Optional context after the span for re-alignment scoring.
 */

/**
 * External request to replace text in Google Docs through the extension content script.
 * @typedef {object} GoogleDocsReplaceTextRequest
 * @property {'replaceText'} kind Request kind.
 * @property {number} start Start offset in bridge text.
 * @property {number} end End offset in bridge text.
 * @property {string} replacementText Replacement text.
 * @property {string=} expectedText Optional expected source text for defensive re-alignment.
 * @property {string=} beforeContext Optional context before the span for re-alignment scoring.
 * @property {string=} afterContext Optional context after the span for re-alignment scoring.
 */

/**
 * All supported bridge requests.
 * @typedef {GoogleDocsGetRectsRequest | GoogleDocsPrepareReplaceTextRequest | GoogleDocsReplaceTextRequest} GoogleDocsRequest
 */

/**
 * Successful response for `getRects`.
 * @typedef {object} GoogleDocsGetRectsResponse
 * @property {'getRects'} kind Response kind.
 * @property {GoogleDocsRect[]} rects Resolved span rects.
 */

/**
 * Successful response for `prepareReplaceText`.
 * @typedef {object} GoogleDocsPrepareReplaceTextResponse
 * @property {'prepareReplaceText'} kind Response kind.
 * @property {boolean} ready Whether the live range was selected and focused.
 * @property {string=} expectedNextText Expected normalized bridge text after the trusted input lands.
 */

/**
 * Successful response for `replaceText`.
 * @typedef {object} GoogleDocsReplaceTextResponse
 * @property {'replaceText'} kind Response kind.
 * @property {boolean} applied Whether the full replacement path completed.
 */

/**
 * Error response for any request.
 * @typedef {object} GoogleDocsErrorResponse
 * @property {'error'} kind Response kind.
 * @property {GoogleDocsRequest['kind']} requestKind Kind of request that failed.
 * @property {string} code Machine-readable error code.
 * @property {string} message Human-readable error message.
 */

/**
 * All bridge responses.
 * @typedef {GoogleDocsGetRectsResponse | GoogleDocsPrepareReplaceTextResponse | GoogleDocsReplaceTextResponse | GoogleDocsErrorResponse} GoogleDocsResponse
 */

/**
 * Notification when bridge text changes.
 * @typedef {object} GoogleDocsTextUpdatedNotification
 * @property {'textUpdated'} kind Notification kind.
 * @property {number} length Current bridge text length.
 */

/**
 * Notification when bridge layout changes.
 * @typedef {object} GoogleDocsLayoutChangedNotification
 * @property {'layoutChanged'} kind Notification kind.
 * @property {number} layoutEpoch Monotonic layout epoch.
 * @property {string} reason Layout change reason.
 */

/**
 * All bridge notifications.
 * @typedef {GoogleDocsTextUpdatedNotification | GoogleDocsLayoutChangedNotification} GoogleDocsNotification
 */

/**
 * Transport envelope for a request event.
 * @typedef {object} GoogleDocsRequestMessage
 * @property {string} protocol Protocol version string.
 * @property {string} requestId Correlation id for request/response pairing.
 * @property {GoogleDocsRequest} request Request payload.
 */

/**
 * Transport envelope for a response event.
 * @typedef {object} GoogleDocsResponseMessage
 * @property {string} protocol Protocol version string.
 * @property {string} requestId Correlation id matching a prior request.
 * @property {GoogleDocsResponse} response Response payload.
 */

/**
 * Transport envelope for a notification event.
 * @typedef {object} GoogleDocsNotificationMessage
 * @property {string} protocol Protocol version string.
 * @property {GoogleDocsNotification} notification Notification payload.
 */

export {};
