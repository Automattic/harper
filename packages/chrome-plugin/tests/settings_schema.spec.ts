import { expect, test } from '@playwright/test';
import {
	DIALECT_NAMES,
	MAX_DELAY,
	MAX_DICTIONARY_WORDS,
	MAX_FILE_BYTES,
	parseSettings,
	SCHEMA_VERSION,
	SettingsParseError,
} from '../src/settings/schema';

function validFile(): Record<string, any> {
	return {
		schema_version: SCHEMA_VERSION,
		exported_at: '2026-10-09T10:00:00.000Z',
		source_app: 'harper-chrome',
		harper_version: '1.0.0',
		core: {
			dialect: 'British',
			lint_config: { SpellCheck: true, LongSentences: false, RepeatedWords: null },
			user_dictionary: ['Harper', 'Elijah'],
			ignored_lints: '{"context_hashes":[18446744073709551615,42]}',
		},
		extension: {
			domain_status: { 'github.com': true, 'mail.google.com': false, localhost: true },
			default_enabled: false,
			delay: 250,
			activation_key: 'shift',
			hotkey: { modifiers: ['Ctrl', 'Shift'], key: 'e' },
			isolate_english: true,
		},
	};
}

function parse(value: unknown) {
	return parseSettings(JSON.stringify(value));
}

test.describe('settings schema', () => {
	test.describe('accepts', () => {
		test('a valid file', () => {
			const parsed = parse(validFile());

			expect(parsed.core.dialect).toBe('British');
			expect(parsed.core.user_dictionary).toEqual(['Harper', 'Elijah']);
			expect(parsed.extension?.delay).toBe(250);
			expect(parsed.extension?.hotkey).toEqual({ modifiers: ['Ctrl', 'Shift'], key: 'e' });
		});

		test('every known dialect', () => {
			for (const dialect of DIALECT_NAMES) {
				const file = validFile();
				file.core.dialect = dialect;

				expect(parse(file).core.dialect).toBe(dialect);
			}
		});

		test('null rule overrides', () => {
			const parsed = parse(validFile());

			expect(parsed.core.lint_config).toEqual({
				SpellCheck: true,
				LongSentences: false,
				RepeatedWords: null,
			});
		});

		test('ignored lint hashes byte-for-byte', () => {
			const parsed = parse(validFile());

			expect(parsed.core.ignored_lints).toBe('{"context_hashes":[18446744073709551615,42]}');
		});

		test('an empty ignored lint list', () => {
			const file = validFile();
			file.core.ignored_lints = '{"context_hashes":[]}';

			expect(parse(file).core.ignored_lints).toBe('{"context_hashes":[]}');
		});

		test('a missing extension block', () => {
			const file = validFile();
			delete file.extension;

			expect(parse(file).extension).toBeUndefined();
		});

		test('a file of exactly MAX_FILE_BYTES', () => {
			const raw = JSON.stringify(validFile());
			const padded = raw + ' '.repeat(MAX_FILE_BYTES - raw.length);

			expect(padded.length).toBe(MAX_FILE_BYTES);
			expect(() => parseSettings(padded)).not.toThrow();
		});

		test('exactly MAX_DICTIONARY_WORDS words', () => {
			const file = validFile();
			file.core.user_dictionary = Array.from({ length: MAX_DICTIONARY_WORDS }, (_, i) => `w${i}`);

			expect(parse(file).core.user_dictionary).toHaveLength(MAX_DICTIONARY_WORDS);
		});

		test('delay at MAX_DELAY, truncating fractions', () => {
			const file = validFile();
			file.extension.delay = MAX_DELAY;
			expect(parse(file).extension?.delay).toBe(MAX_DELAY);

			file.extension.delay = 1.9;
			expect(parse(file).extension?.delay).toBe(1);
		});

		test('empty hotkey modifiers', () => {
			const file = validFile();
			file.extension.hotkey = { modifiers: [], key: 'e' };

			expect(parse(file).extension?.hotkey.modifiers).toEqual([]);
		});
	});

	test.describe('drops / sanitises', () => {
		test('unknown top-level blocks and fields', () => {
			const file = validFile();
			file.desktop = { integrations: ['com.apple.TextEdit'] };
			file.core.something_new = 1;

			const parsed = parse(file) as Record<string, any>;

			expect(parsed.desktop).toBeUndefined();
			expect(parsed.core.something_new).toBeUndefined();
		});

		test('__proto__ keys stay own properties and do not pollute', () => {
			const raw = JSON.stringify(validFile()).replace('"SpellCheck":true', '"__proto__":true');
			const cfg = parseSettings(raw).core.lint_config;

			expect(Object.getPrototypeOf(cfg)).toBe(Object.prototype);
			expect(Object.keys(cfg)).toContain('__proto__');
			expect(({} as Record<string, unknown>).polluted).toBeUndefined();
			expect((Object.prototype as Record<string, unknown>).SpellCheck).toBeUndefined();
		});
	});

	test.describe('rejects: envelope', () => {
		test('non-string input', () => {
			expect(() => parseSettings(42 as unknown as string)).toThrow(/must be text/);
		});

		test('malformed JSON', () => {
			expect(() => parseSettings('{"schema_version": 1,')).toThrow(/not valid JSON/);
		});

		test('non-object root', () => {
			for (const raw of ['null', '[]', '1', '"x"']) {
				expect(() => parseSettings(raw)).toThrow(SettingsParseError);
			}
		});

		test('oversized files', () => {
			const raw = ' '.repeat(MAX_FILE_BYTES + 1);

			expect(() => parseSettings(raw)).toThrow(/larger than/);
		});

		test('a newer schema version', () => {
			const file = validFile();
			file.schema_version = SCHEMA_VERSION + 1;

			expect(() => parse(file)).toThrow(/schema_version/);
		});

		test('missing or invalid schema version', () => {
			const missing = validFile();
			delete missing.schema_version;
			expect(() => parse(missing)).toThrow(SettingsParseError);

			for (const bad of [0, -1, 1.5, '1', null]) {
				const file = validFile();
				file.schema_version = bad;

				expect(() => parse(file)).toThrow(/schema_version/);
			}
		});

		test('overlong metadata strings', () => {
			for (const field of ['exported_at', 'source_app', 'harper_version']) {
				const file = validFile();
				file[field] = 'x'.repeat(257);

				expect(() => parse(file)).toThrow(new RegExp(field));
			}
		});

		test('a missing core block', () => {
			const file = validFile();
			delete file.core;

			expect(() => parse(file)).toThrow(/core/);
		});
	});

	test.describe('rejects: core', () => {
		test('an unknown dialect', () => {
			for (const bad of [0, 'Klingon', 'british', null]) {
				const file = validFile();
				file.core.dialect = bad;

				expect(() => parse(file)).toThrow(/dialect/);
			}
		});

		test('non-boolean rule values', () => {
			const file = validFile();
			file.core.lint_config = { SpellCheck: 'yes' };

			expect(() => parse(file)).toThrow(/SpellCheck/);
		});

		test('bad rule names', () => {
			for (const name of ['', 'a'.repeat(129)]) {
				const file = validFile();
				file.core.lint_config = { [name]: true };

				expect(() => parse(file)).toThrow(/rule name/);
			}
		});

		test('too many dictionary words', () => {
			const file = validFile();
			file.core.user_dictionary = Array.from(
				{ length: MAX_DICTIONARY_WORDS + 1 },
				(_, i) => `w${i}`,
			);

			expect(() => parse(file)).toThrow(/more than/);
		});

		test('a non-array dictionary', () => {
			const file = validFile();
			file.core.user_dictionary = 'Harper';

			expect(() => parse(file)).toThrow(/user_dictionary/);
		});

		test('bad dictionary words', () => {
			for (const bad of [42, '', '   ', 'a'.repeat(129), 'tab\there']) {
				const file = validFile();
				file.core.user_dictionary = [bad];

				expect(() => parse(file)).toThrow(/user_dictionary/);
			}
		});

		test('bad ignored_lints values', () => {
			for (const bad of [
				'[1,2,3]',
				'{"context_hashes":[1.5]}',
				'{"context_hashes":[-1]}',
				'{"context_hashes":["1"]}',
				'{"context_hashes":[18446744073709551616]}', // u64::MAX + 1
				'{"context_hashes":[100000000000000000000]}', // 21 digits
				{ context_hashes: [1] },
			]) {
				const file = validFile();
				file.core.ignored_lints = bad;

				expect(() => parse(file)).toThrow(/ignored_lints/);
			}
		});
	});

	test.describe('rejects: extension', () => {
		test('a partial extension block', () => {
			const file = validFile();
			file.extension = { default_enabled: true };

			expect(() => parse(file)).toThrow(SettingsParseError);
		});

		test('bad extension values', () => {
			const cases: [string, unknown][] = [
				['default_enabled', 'yes'],
				['delay', -1],
				['delay', MAX_DELAY + 1],
				['delay', '250'],
				['activation_key', 'meta'],
				['isolate_english', 1],
				['hotkey', { modifiers: ['Cmd'], key: 'e' }],
				['domain_status', { 'bad domain': true }],
				['domain_status', { 'github.com': 'yes' }],
			];

			for (const [field, value] of cases) {
				const file = validFile();
				file.extension[field] = value;

				expect(() => parse(file), `${field}=${JSON.stringify(value)}`).toThrow(SettingsParseError);
			}
		});

		test('bad hotkeys', () => {
			const cases: [string, unknown][] = [
				['missing modifiers', { key: 'e' }],
				['non-array modifiers', { modifiers: 'Ctrl', key: 'e' }],
				['duplicate modifiers', { modifiers: ['Ctrl', 'Ctrl'], key: 'e' }],
				['empty key', { modifiers: ['Ctrl'], key: '' }],
				['long key', { modifiers: ['Ctrl'], key: 'k'.repeat(33) }],
				['control char key', { modifiers: ['Ctrl'], key: 'e\n' }],
				['non-string key', { modifiers: ['Ctrl'], key: 5 }],
			];

			for (const [label, hotkey] of cases) {
				const file = validFile();
				file.extension.hotkey = hotkey;

				expect(() => parse(file), label).toThrow(/hotkey/);
			}
		});
	});
});
