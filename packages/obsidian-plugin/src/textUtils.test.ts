import { binary, LocalLinter } from 'harper.js';
import { describe, expect, test } from 'vitest';
import { charIndexToCodeUnit, linesToString, stringToLines } from './textUtils';

test('Dictionary values are reversible', () => {
	const possibleDicts = [
		[
			'lynx',
			'capybara',
			'ibex',
			'wombat',
			'ocelot',
			'pangolin',
			'stoat',
			'vole',
			'caracal',
			'gazelle',
		],
		[
			'azurite',
			'feldspar',
			'gabbro',
			'peridot',
			'chalcedony',
			'rutile',
			'aragonite',
			'spinel',
			'pyrite',
			'malachite',
		],
		[
			'auscultation',
			'phlebotomy',
			'sutures',
			'anticoagulant',
			'intubation',
			'tachycardia',
			'catheter',
			'defibrillator',
			'ischemia',
			'hematoma',
		],
		[
			'fennel',
			'sunchoke',
			'burrata',
			'tamarind',
			'sumac',
			'cassava',
			'farro',
			'durian',
			'romanesco',
			'chicory',
		],
		[
			'taciturn',
			'indelible',
			'verdant',
			'oblique',
			'incisive',
			'mellifluous',
			'crepuscular',
			'effulgent',
			'sinistral',
			'pellucid',
		],
	];

	for (const set of possibleDicts) {
		const text = linesToString(set);
		const back = stringToLines(text);

		expect(back).toStrictEqual(set);
	}
});

test('Can handle multiple newlines', () => {
	const dictText = 'worda\n\nwordb';

	expect(stringToLines(dictText)).toStrictEqual(['worda', 'wordb']);
});

test('Can handle carriage returns', () => {
	const dictText = 'worda\r\n\r\nwordb\r\nwordc';
	expect(stringToLines(dictText)).toStrictEqual(['worda', 'wordb', 'wordc']);
});

test('handles output from Harper', async () => {
	const text = '✉️👋👍✉️🚀✉️🌴 This is to show the offset issue sdssda is it there?';

	const linter = new LocalLinter({ binary });
	const lints = await linter.lint(text);

	const span = lints[0].span();
	const chars = Array.from(text);

	expect(charIndexToCodeUnit(span.start, chars)).toBe(48);
	expect(charIndexToCodeUnit(span.end, chars)).toBe(54);
});

test('charIndexToCodeUnit handles zero index', () => {
	const text = 'abc';
	const chars = Array.from(text);

	expect(charIndexToCodeUnit(0, chars)).toBe(0);
});

test('charIndexToCodeUnit sums ASCII code units', () => {
	const text = 'harper';
	const chars = Array.from(text);

	expect(charIndexToCodeUnit(3, chars)).toBe(3);
	expect(charIndexToCodeUnit(chars.length, chars)).toBe(text.length);
});

test('charIndexToCodeUnit handles mixed emoji and text', () => {
	const text = '👋hi👍';
	const chars = Array.from(text);

	expect(chars).toStrictEqual(['👋', 'h', 'i', '👍']);
	expect(charIndexToCodeUnit(1, chars)).toBe(2);
	expect(charIndexToCodeUnit(3, chars)).toBe(4);
	expect(charIndexToCodeUnit(4, chars)).toBe(6);
});

test('charIndexToCodeUnit handles regional indicator pairs', () => {
	const text = '🇨🇦ca';
	const chars = Array.from(text);

	expect(chars).toStrictEqual(['🇨', '🇦', 'c', 'a']);
	expect(charIndexToCodeUnit(2, chars)).toBe(4);
	expect(charIndexToCodeUnit(4, chars)).toBe(6);
});

test('charIndexToCodeUnit handles variation selectors', () => {
	const text = '✉️✉️';
	const chars = Array.from(text);

	expect(chars).toStrictEqual(['✉', '️', '✉', '️']);
	expect(charIndexToCodeUnit(1, chars)).toBe(1);
	expect(charIndexToCodeUnit(2, chars)).toBe(2);
	expect(charIndexToCodeUnit(3, chars)).toBe(3);
	expect(charIndexToCodeUnit(4, chars)).toBe(4);
});
