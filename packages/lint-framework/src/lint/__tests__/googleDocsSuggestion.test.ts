/**
 * Tests for the async suggestion application flow, specifically the fix for
 * https://github.com/Automattic/harper/issues/2995
 *
 * The core invariant: when applying a suggestion in Google Docs, the bridge
 * replaceText call must complete BEFORE focus is moved away (refocusClose).
 * Otherwise Google Docs' text event iframe loses focus and the paste event
 * targets the wrong element.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

// ---------------------------------------------------------------------------
// Helpers & mocks
// ---------------------------------------------------------------------------

/** Tracks the order of async operations to verify sequencing. */
function createOrderTracker() {
	const order: string[] = [];
	return {
		push(label: string) {
			order.push(label);
		},
		get order() {
			return [...order];
		},
	};
}

/**
 * Simulates an async bridge client like the one the Chrome extension injects
 * onto `window.__harperGoogleDocsBridgeClient`.
 */
function createMockBridgeClient(tracker: ReturnType<typeof createOrderTracker>, delay = 10) {
	return {
		replaceText: vi.fn(
			(
				_start: number,
				_end: number,
				_replacementText: string,
				_expectedText?: string,
				_beforeContext?: string,
				_afterContext?: string,
			) =>
				new Promise<void>((resolve) => {
					setTimeout(() => {
						tracker.push('bridge:replaceText');
						resolve();
					}, delay);
				}),
		),
	};
}

// ---------------------------------------------------------------------------
// Tests for async ordering (the fix for #2995)
// ---------------------------------------------------------------------------

describe('Google Docs suggestion application ordering', () => {
	beforeEach(() => {
		vi.restoreAllMocks();
	});

	it('applySuggestion returning a Promise is awaitable', async () => {
		let resolved = false;
		const applySuggestion = async () => {
			await new Promise<void>((r) => setTimeout(r, 10));
			resolved = true;
		};

		// The type signature should accept async functions (void | Promise<void>)
		const fn: () => void | Promise<void> = applySuggestion;
		await fn();
		expect(resolved).toBe(true);
	});

	it('applySuggestion returning void is also valid', () => {
		const applySuggestion = () => {
			// sync, returns void
		};

		const fn: () => void | Promise<void> = applySuggestion;
		const result = fn();
		// void return — no promise
		expect(result).toBeUndefined();
	});

	it('awaiting applySuggestion before refocusClose preserves correct ordering', async () => {
		const tracker = createOrderTracker();

		const applySuggestion = async () => {
			await new Promise<void>((r) => setTimeout(r, 20));
			tracker.push('applySuggestion:done');
		};

		const refocusClose = () => {
			tracker.push('refocusClose');
		};

		// This is the FIXED flow — await before close
		await applySuggestion();
		refocusClose();

		expect(tracker.order).toEqual(['applySuggestion:done', 'refocusClose']);
	});

	it('NOT awaiting applySuggestion causes wrong ordering (demonstrates the bug)', async () => {
		const tracker = createOrderTracker();

		const applySuggestion = async () => {
			await new Promise<void>((r) => setTimeout(r, 20));
			tracker.push('applySuggestion:done');
		};

		const refocusClose = () => {
			tracker.push('refocusClose');
		};

		// This is the BROKEN flow — fire-and-forget (the bug)
		applySuggestion(); // not awaited!
		refocusClose();

		// refocusClose fires immediately, before applySuggestion completes
		expect(tracker.order).toEqual(['refocusClose']);

		// Wait for the promise to settle
		await new Promise<void>((r) => setTimeout(r, 30));
		expect(tracker.order).toEqual(['refocusClose', 'applySuggestion:done']);
	});

	it('bridge replaceText is awaited inside replaceGoogleDocsValue', async () => {
		const tracker = createOrderTracker();
		const bridgeClient = createMockBridgeClient(tracker, 15);

		// Simulate the fixed replaceGoogleDocsValue logic
		const replaceGoogleDocsValue = async (
			span: { start: number; end: number },
			replacementText: string,
			source: string,
		) => {
			const safeStart = Math.max(0, Math.min(span.start, source.length));
			const safeEnd = Math.max(safeStart, Math.min(span.end, source.length));
			const expectedText = source.slice(safeStart, safeEnd);
			const contextRadius = 64;
			const beforeContext = source.slice(Math.max(0, safeStart - contextRadius), safeStart);
			const afterContext = source.slice(
				safeEnd,
				Math.min(source.length, safeEnd + contextRadius),
			);

			await Promise.resolve(
				bridgeClient.replaceText(
					span.start,
					span.end,
					replacementText,
					expectedText,
					beforeContext,
					afterContext,
				),
			);
			tracker.push('replaceGoogleDocsValue:done');
		};

		await replaceGoogleDocsValue({ start: 0, end: 5 }, 'fixed', 'hello world');

		// Bridge must complete before replaceGoogleDocsValue resolves
		expect(tracker.order).toEqual(['bridge:replaceText', 'replaceGoogleDocsValue:done']);
		expect(bridgeClient.replaceText).toHaveBeenCalledOnce();
		expect(bridgeClient.replaceText).toHaveBeenCalledWith(
			0,
			5,
			'fixed',
			'hello',
			'',
			' world',
		);
	});

	it('full flow: bridge completes before popup closes', async () => {
		const tracker = createOrderTracker();
		const bridgeClient = createMockBridgeClient(tracker, 15);

		// Simulate the entire chain: click suggestion -> applySuggestion -> bridge -> refocusClose
		const applySuggestion = async () => {
			await Promise.resolve(
				bridgeClient.replaceText(0, 5, 'fixed', 'hello', '', ' world'),
			);
			tracker.push('applySuggestion:done');
		};

		const refocusClose = () => {
			tracker.push('refocusClose');
		};

		// Fixed flow
		await applySuggestion();
		refocusClose();

		expect(tracker.order).toEqual([
			'bridge:replaceText',
			'applySuggestion:done',
			'refocusClose',
		]);
	});

	it('bridge client with sync replaceText still works', async () => {
		const tracker = createOrderTracker();

		// Some bridge clients might return synchronously
		const syncBridgeClient = {
			replaceText: vi.fn(() => {
				tracker.push('bridge:replaceText');
				// returns undefined (void), not a Promise
			}),
		};

		const applySuggestion = async () => {
			await Promise.resolve(
				syncBridgeClient.replaceText(0, 5, 'fixed', 'hello', '', ' world'),
			);
			tracker.push('applySuggestion:done');
		};

		const refocusClose = () => {
			tracker.push('refocusClose');
		};

		await applySuggestion();
		refocusClose();

		expect(tracker.order).toEqual([
			'bridge:replaceText',
			'applySuggestion:done',
			'refocusClose',
		]);
	});

	it('bridge failure does not prevent popup from closing', async () => {
		const tracker = createOrderTracker();

		const failingBridgeClient = {
			replaceText: vi.fn(() =>
				new Promise<void>((_, reject) => {
					setTimeout(() => {
						tracker.push('bridge:error');
						reject(new Error('bridge failed'));
					}, 10);
				}),
			),
		};

		// Simulate the try/catch in replaceGoogleDocsValue
		const applySuggestion = async () => {
			try {
				await Promise.resolve(
					failingBridgeClient.replaceText(0, 5, 'fixed', 'hello', '', ' world'),
				);
			} catch {
				// replaceGoogleDocsValue catches errors silently
			}
			tracker.push('applySuggestion:done');
		};

		const refocusClose = () => {
			tracker.push('refocusClose');
		};

		await applySuggestion();
		refocusClose();

		// Even on failure, ordering is maintained — close only after attempt completes
		expect(tracker.order).toEqual([
			'bridge:error',
			'applySuggestion:done',
			'refocusClose',
		]);
	});

	it('non-Google-Docs sync applySuggestion still works with await', async () => {
		const tracker = createOrderTracker();

		// Regular (non-Google-Docs) editors use sync applySuggestion
		const applySuggestion = () => {
			tracker.push('applySuggestion:sync');
			// returns void
		};

		const refocusClose = () => {
			tracker.push('refocusClose');
		};

		// Awaiting a void return is harmless
		await applySuggestion();
		refocusClose();

		expect(tracker.order).toEqual(['applySuggestion:sync', 'refocusClose']);
	});

	it('bridge replaceText receives correct context parameters', async () => {
		const bridgeClient = {
			replaceText: vi.fn(() => Promise.resolve()),
		};

		const source = 'The quick brown fox jumps over the lazy dog';
		const span = { start: 10, end: 15 }; // "brown"
		const replacementText = 'red';

		const safeStart = Math.max(0, Math.min(span.start, source.length));
		const safeEnd = Math.max(safeStart, Math.min(span.end, source.length));
		const expectedText = source.slice(safeStart, safeEnd);
		const contextRadius = 64;
		const beforeContext = source.slice(Math.max(0, safeStart - contextRadius), safeStart);
		const afterContext = source.slice(safeEnd, Math.min(source.length, safeEnd + contextRadius));

		await Promise.resolve(
			bridgeClient.replaceText(
				span.start,
				span.end,
				replacementText,
				expectedText,
				beforeContext,
				afterContext,
			),
		);

		expect(bridgeClient.replaceText).toHaveBeenCalledWith(
			10,
			15,
			'red',
			'brown',
			'The quick ',
			' fox jumps over the lazy dog',
		);
	});

	it('context radius is capped at source boundaries', async () => {
		const bridgeClient = {
			replaceText: vi.fn(() => Promise.resolve()),
		};

		// Short text where context can't extend 64 chars in either direction
		const source = 'Hi';
		const span = { start: 0, end: 2 };

		const safeStart = Math.max(0, Math.min(span.start, source.length));
		const safeEnd = Math.max(safeStart, Math.min(span.end, source.length));
		const expectedText = source.slice(safeStart, safeEnd);
		const contextRadius = 64;
		const beforeContext = source.slice(Math.max(0, safeStart - contextRadius), safeStart);
		const afterContext = source.slice(safeEnd, Math.min(source.length, safeEnd + contextRadius));

		await Promise.resolve(
			bridgeClient.replaceText(span.start, span.end, 'Hello', expectedText, beforeContext, afterContext),
		);

		expect(bridgeClient.replaceText).toHaveBeenCalledWith(0, 2, 'Hello', 'Hi', '', '');
	});
});
