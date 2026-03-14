import { expect, test } from 'vitest';
import {
	cloneCustomReplacements,
	customReplacementsToString,
	getCustomSuggestions,
	normalizeCustomReplacements,
	stringToCustomReplacements,
} from './customSuggestions';

test('parses custom suggestions from textarea format', () => {
	expect(
		stringToCustomReplacements('adn -> and\nteh => the, tech\ninvalid\nmissing -> '),
	).toStrictEqual({
		adn: ['and'],
		teh: ['the', 'tech'],
	});
});

test('serializes custom suggestions to textarea format', () => {
	expect(
		customReplacementsToString({
			adn: ['and'],
			teh: ['the', 'tech'],
		}),
	).toBe('adn -> and\nteh -> the, tech');
});

test('normalizes keys and removes duplicate suggestions', () => {
	expect(
		normalizeCustomReplacements({
			' ADN ': ['and', 'And', ''],
			'': ['ignored'],
		}),
	).toStrictEqual({
		adn: ['and'],
	});
});

test('adapts custom suggestions to uppercase words', () => {
	expect(getCustomSuggestions('ADN', { adn: ['and'] })).toStrictEqual(['AND']);
});

test('adapts custom suggestions to title case words', () => {
	expect(getCustomSuggestions('Adn', { adn: ['and'] })).toStrictEqual(['And']);
});

test('keeps custom suggestions lowercase by default', () => {
	expect(getCustomSuggestions('adn', { adn: ['and'] })).toStrictEqual(['and']);
});

test('clones custom suggestion maps deeply', () => {
	const original = { adn: ['and'] };
	const cloned = cloneCustomReplacements(original);
	cloned?.adn.push('add');

	expect(cloned).toStrictEqual({ adn: ['and', 'add'] });
	expect(original).toStrictEqual({ adn: ['and'] });
});
