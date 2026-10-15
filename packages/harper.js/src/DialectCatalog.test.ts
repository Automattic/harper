import { expect, test } from 'vitest';
import { binary } from './binaries/binary';
import { Dialect } from './main';

test('The dialect catalog lists English first, with names and codes', async () => {
	const catalog = await binary.getDialectCatalog();

	expect(catalog[0]).toEqual({
		dialect: Dialect.American,
		language: 'English',
		region: 'American',
		code: 'US',
	});
	expect(catalog.map((info) => info.code)).toContain('GB');
});

test('Every dialect in the catalog is a Dialect value, listed once', async () => {
	const catalog = await binary.getDialectCatalog();
	const dialects = catalog.map((info) => info.dialect);

	for (const dialect of dialects) {
		expect(Dialect[dialect]).toBeTypeOf('string');
	}
	expect(new Set(dialects).size).toBe(dialects.length);
});
