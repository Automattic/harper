import { strToU8, zipSync } from 'fflate';
import { describe, expect, test } from 'vitest';
import { packWeirpackFiles, unpackWeirpackBytes } from './weirpack';

describe('weirpack helpers', () => {
	test('round-trips a weirpack archive', () => {
		const manifest = {
			author: 'Test Author',
			version: '0.1.0',
			description: 'Test pack',
			license: 'MIT',
		};

		const files = new Map([
			['manifest.json', JSON.stringify(manifest, null, 2)],
			['ExampleRule.weir', 'expr main test'],
			['AnotherRule.weir', 'expr main banana'],
		]);

		const bytes = packWeirpackFiles(files);
		const unpacked = unpackWeirpackBytes(bytes);

		expect(unpacked.manifest).toEqual(manifest);
		expect(unpacked.files.get('ExampleRule.weir')).toBe('expr main test');
		expect(unpacked.files.get('AnotherRule.weir')).toBe('expr main banana');
	});

	test('packWeirpackFiles requires a manifest.json file', () => {
		expect(() =>
			packWeirpackFiles(new Map([['Rule.weir', 'expr main test']])),
		).toThrow('Weirpack is missing manifest.json');
	});

	test('unpackWeirpackBytes requires a manifest.json file', () => {
		const bytes = zipSync({
			'Rule.weir': strToU8('expr main test'),
		});

		expect(() => unpackWeirpackBytes(bytes)).toThrow('Weirpack is missing manifest.json');
	});
});
