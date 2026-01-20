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

		const files = [
			{
				name: 'manifest.json',
				content: JSON.stringify(manifest, null, 2),
			},
			{
				name: 'ExampleRule.weir',
				content: 'expr main test',
			},
			{
				name: 'AnotherRule.weir',
				content: 'expr main banana',
			},
		];

		const bytes = packWeirpackFiles(files);
		const unpacked = unpackWeirpackBytes(bytes);

		expect(unpacked.manifest).toEqual(manifest);
		expect(unpacked.rules).toEqual([
			{
				name: 'AnotherRule.weir',
				content: 'expr main banana',
			},
			{
				name: 'ExampleRule.weir',
				content: 'expr main test',
			},
		]);
	});

	test('packWeirpackFiles requires a manifest.json file', () => {
		expect(() =>
			packWeirpackFiles([
				{
					name: 'Rule.weir',
					content: 'expr main test',
				},
			]),
		).toThrow('Weirpack is missing manifest.json');
	});

	test('unpackWeirpackBytes requires a manifest.json file', () => {
		const bytes = zipSync({
			'Rule.weir': strToU8('expr main test'),
		});

		expect(() => unpackWeirpackBytes(bytes)).toThrow('Weirpack is missing manifest.json');
	});
});
