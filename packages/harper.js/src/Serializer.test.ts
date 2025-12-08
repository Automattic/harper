import { Span } from 'harper-wasm';
import { beforeEach, describe, expect, test } from 'vitest';
import { binary } from './binary';
import LocalLinter from './LocalLinter';
import Serializer from './Serializer';

describe('Serializer', () => {
	let serializer = new Serializer(binary);

	beforeEach(() => {
		serializer = new Serializer(binary);
	});

	test('works with strings', async () => {
		const start = 'This is a string';

		const end = await serializer.deserializeArg(
			structuredClone(await serializer.serializeArg(start)),
		);

		expect(end).toBe(start);
		expect(typeof end).toBe(typeof start);
	});

	test('works with false booleans', async () => {
		const start = false;

		const end = await serializer.deserializeArg(
			structuredClone(await serializer.serializeArg(start)),
		);

		expect(end).toBe(start);
		expect(typeof end).toBe(typeof start);
	});

	test('works with true booleans', async () => {
		const start = true;

		const end = await serializer.deserializeArg(
			structuredClone(await serializer.serializeArg(start)),
		);

		expect(end).toBe(start);
		expect(typeof end).toBe(typeof start);
	});

	test('works with numbers', async () => {
		const start = 123;

		const end = await serializer.deserializeArg(
			structuredClone(await serializer.serializeArg(start)),
		);

		expect(end).toBe(start);
		expect(typeof end).toBe(typeof start);
	});

	test('works with Spans', async () => {
		const start = Span.new(123, 321);

		const end = await serializer.deserializeArg(
			structuredClone(await serializer.serializeArg(start)),
		);

		expect(end.start).toBe(start.start);
		expect(end.len()).toBe(start.len());
		expect(typeof end).toBe(typeof start);
	});

	test('works with Lints', async () => {
		const linter = new LocalLinter({ binary });
		const lints = await linter.lint('This is an test.');
		const start = lints[0];

		expect(start).not.toBeNull();

		const end = await serializer.deserializeArg(
			structuredClone(await serializer.serializeArg(start)),
		);

		expect(end.message()).toBe(start.message());
		expect(end.lint_kind()).toBe(start.lint_kind());
	});
});
