import { binary, LocalLinter } from 'harper.js';
import { describe, expect, test } from 'vitest';
import { linesToString, stringToLines } from './textUtils';

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
