import { afterEach, describe, expect, it } from 'vitest';
import LintFramework from './LintFramework';
import type { UnpackedLintGroups } from './unpackLint';

/**
 * `LintFramework` polls itself every second to cover editors that fail to emit
 * events, so that poll can supply a lint the framework never scheduled itself.
 * Every wait here is bounded by wall-clock time well below one second, and the
 * tests additionally assert on *when* a follow-up arrived. A lint produced by
 * the poll is a second late by construction and cannot satisfy that, however
 * slowly the machine happens to be running.
 *
 * Frame counting was tried first and is not good enough: `requestAnimationFrame`
 * is throttled when a page is backgrounded and stretches under load, so a fixed
 * number of frames can silently exceed the poll interval. That turns a broken
 * scheduler into a passing test, which is worse than a flake.
 *
 * This value MUST stay below the poll interval in `LintFramework`'s
 * constructor. Raising it above one second is what would let the poll satisfy
 * these tests, quietly restoring the failure mode described above.
 */
const SETTLE_BUDGET_MS = 400;

/** Let queued microtasks and one animation frame run. */
async function tick() {
	await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
	await Promise.resolve();
}

/** Wait for `count` lint requests, giving up well before the one-second poll. */
async function waitForCalls(calls: string[], count: number) {
	const deadline = performance.now() + SETTLE_BUDGET_MS;
	while (calls.length < count && performance.now() < deadline) {
		await tick();
	}
}

/** Give the framework room to act when we are asserting that it does not. */
async function quietPeriod() {
	const deadline = performance.now() + SETTLE_BUDGET_MS / 2;
	while (performance.now() < deadline) {
		await tick();
	}
}

/**
 * A lint provider whose responses are resolved by hand, so a test can hold a
 * lint "in flight" and decide exactly when it completes.
 */
function deferredProvider() {
	const calls: string[] = [];
	/** When each entry in `calls` was requested, for asserting promptness. */
	const callTimes: number[] = [];
	const pending: { resolve: () => void; reject: (reason: Error) => void }[] = [];

	const provider = (text: string): Promise<UnpackedLintGroups> => {
		calls.push(text);
		callTimes.push(performance.now());
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
		callTimes,
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
		const { provider, calls, callTimes, resolveNext } = deferredProvider();
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
		await quietPeriod();
		expect(calls).toEqual(['T']);

		const releasedAt = performance.now();
		await resolveNext();
		await waitForCalls(calls, 2);

		// Exactly one follow-up, seeing the text as it now stands.
		expect(calls).toEqual(['T', 'This is a mistaek.']);

		// And issued off the back of the pass completing, not by the one-second
		// poll -- which could not have produced it this quickly.
		expect(callTimes[1] - releasedAt).toBeLessThan(SETTLE_BUDGET_MS);
	});

	it('still issues the queued follow-up when the in-flight pass rejects', async () => {
		const editor = makeTextarea('T');
		const { provider, calls, callTimes, rejectNext } = deferredProvider();
		const fw = new LintFramework(provider, {});

		try {
			window.addEventListener('unhandledrejection', suppressProviderFailure);

			await fw.addTarget(editor);
			await waitForCalls(calls, 1);
			expect(calls).toEqual(['T']);

			// Input arrives mid-pass and is coalesced into a follow-up.
			editor.value = 'This is a mistaek.';
			fw.update();
			await quietPeriod();
			expect(calls).toEqual(['T']);

			// The pass then fails. Releasing the guard is not enough on its own: the
			// queued work has to be handed off too, or the input that arrived during
			// a failed lint is silently forgotten until the one-second poll.
			const releasedAt = performance.now();
			await rejectNext();
			await waitForCalls(calls, 2);

			expect(calls).toEqual(['T', 'This is a mistaek.']);
			expect(callTimes[1] - releasedAt).toBeLessThan(SETTLE_BUDGET_MS);
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
