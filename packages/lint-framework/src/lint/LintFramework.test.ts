import { afterEach, describe, expect, it } from 'vitest';
import LintFramework from './LintFramework';
import type { UnpackedLintGroups } from './unpackLint';

/**
 * `LintFramework` polls itself every second to cover editors that fail to emit
 * events. Every wait here stays far below that, so a passing assertion means
 * the framework re-linted deliberately rather than being rescued by the poll.
 */
const TICKS = 10;

/** Let queued microtasks and one animation frame run. */
async function tick() {
	await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
	await Promise.resolve();
}

async function waitTicks(count = TICKS) {
	for (let i = 0; i < count; i++) {
		await tick();
	}
}

/** Wait for `count` lint requests, giving up well before the one-second poll. */
async function waitForCalls(calls: string[], count: number) {
	for (let i = 0; i < TICKS && calls.length < count; i++) {
		await tick();
	}
}

/**
 * A lint provider whose responses are resolved by hand, so a test can hold a
 * lint "in flight" and decide exactly when it completes.
 */
function deferredProvider() {
	const calls: string[] = [];
	const pending: { resolve: () => void; reject: (reason: Error) => void }[] = [];

	const provider = (text: string): Promise<UnpackedLintGroups> => {
		calls.push(text);
		return new Promise((resolve, reject) => {
			pending.push({ resolve: () => resolve({}), reject });
		});
	};

	async function settleOldest(outcome: 'resolve' | 'reject') {
		const next = pending.shift();
		if (next == null) {
			throw new Error('No outstanding lint request to settle.');
		}

		if (outcome === 'resolve') {
			next.resolve();
		} else {
			next.reject(new Error(PROVIDER_FAILURE));
		}

		await tick();
	}

	return {
		provider,
		calls,
		/** Complete the oldest outstanding request. */
		resolveNext: () => settleOldest('resolve'),
		/** Fail the oldest outstanding request. */
		rejectNext: () => settleOldest('reject'),
	};
}

const PROVIDER_FAILURE = 'lint provider exploded';

/**
 * `update()` calls `requestLintUpdate()` without awaiting it, so a rejected
 * lint surfaces as an unhandled rejection rather than reaching a caller. That
 * is existing behaviour and out of scope here; swallow the one failure this
 * test provokes so it does not read as an unrelated error.
 */
function suppressProviderFailure(event: PromiseRejectionEvent) {
	if (event.reason instanceof Error && event.reason.message === PROVIDER_FAILURE) {
		event.preventDefault();
	}
}

const targets: HTMLTextAreaElement[] = [];

function makeTextarea(value: string): HTMLTextAreaElement {
	const el = document.createElement('textarea');
	el.value = value;
	el.style.width = '400px';
	el.style.height = '100px';
	document.body.appendChild(el);
	targets.push(el);
	return el;
}

afterEach(() => {
	for (const el of targets.splice(0)) {
		el.remove();
	}
});

describe('LintFramework lint scheduling', () => {
	it('re-lints with the final text as soon as the in-flight pass finishes', async () => {
		const editor = makeTextarea('T');
		const { provider, calls, resolveNext } = deferredProvider();
		const fw = new LintFramework(provider, {});

		await fw.addTarget(editor);
		await waitForCalls(calls, 1);
		expect(calls).toEqual(['T']);

		// Further input arrives while the first lint is still in flight. The
		// single-flight guard drops each of these.
		editor.value = 'This is a mistaek.';
		fw.update();
		fw.update();
		fw.update();
		await waitTicks();
		expect(calls).toEqual(['T']);

		await resolveNext();
		await waitForCalls(calls, 2);

		// Exactly one follow-up, seeing the text as it now stands.
		expect(calls).toEqual(['T', 'This is a mistaek.']);
	});

	it('still issues the queued follow-up when the in-flight pass rejects', async () => {
		const editor = makeTextarea('T');
		const { provider, calls, rejectNext } = deferredProvider();
		const fw = new LintFramework(provider, {});

		try {
			window.addEventListener('unhandledrejection', suppressProviderFailure);

			await fw.addTarget(editor);
			await waitForCalls(calls, 1);
			expect(calls).toEqual(['T']);

			// Input arrives mid-pass and is coalesced into a follow-up.
			editor.value = 'This is a mistaek.';
			fw.update();
			await waitTicks();
			expect(calls).toEqual(['T']);

			// The pass then fails. Releasing the guard is not enough on its own: the
			// queued work has to be handed off too, or the input that arrived during
			// a failed lint is silently forgotten until the one-second poll.
			await rejectNext();
			await waitForCalls(calls, 2);

			expect(calls).toEqual(['T', 'This is a mistaek.']);
		} finally {
			window.removeEventListener('unhandledrejection', suppressProviderFailure);
		}
	});

	it('releases the in-flight guard when a lint request rejects', async () => {
		const editor = makeTextarea('First.');
		const calls: string[] = [];
		let failNext = true;

		const provider = async (text: string): Promise<UnpackedLintGroups> => {
			calls.push(text);
			if (failNext) {
				failNext = false;
				throw new Error(PROVIDER_FAILURE);
			}
			return {};
		};

		const fw = new LintFramework(provider, {});

		try {
			window.addEventListener('unhandledrejection', suppressProviderFailure);

			await fw.addTarget(editor);
			await waitForCalls(calls, 1);
			expect(calls[0]).toBe('First.');

			// A rejection must not leave the framework permanently wedged.
			editor.value = 'Second.';
			fw.update();
			await waitForCalls(calls, 2);

			// Assert on order, not on an exact count. Ambient page events -- the
			// window listeners cover scroll, resize and selectionchange -- can
			// legitimately queue further passes, and a coalesced follow-up may add
			// one more. What matters is that the failed pass was followed by one
			// seeing the new text.
			expect(calls.slice(0, 2)).toEqual(['First.', 'Second.']);
		} finally {
			window.removeEventListener('unhandledrejection', suppressProviderFailure);
		}
	});
});
