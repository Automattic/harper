import { expect, test } from 'vitest';
import { dictToString, stringToDict } from './dictUtils';

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
		const text = dictToString(set);
		const back = stringToDict(text);

		expect(back).toStrictEqual(set);
	}
});

test('Can handle multiple newlines', () => {
	const dictText = 'worda\n\nwordb';

	expect(stringToDict(dictText)).toStrictEqual(['worda', 'wordb']);
});

test('Can handle carriage returns', () => {
	const dictText = 'worda\r\n\r\nwordb\r\nwordc';
	expect(stringToDict(dictText)).toStrictEqual(['worda', 'wordb', 'wordc']);
});
