import { expect, test } from '@playwright/test';
import {
	buildExport,
	type CurrentSettings,
	computeImportedSettings,
	extractIgnoredLintHashes,
	planImport,
	summarizeImport,
} from '../src/settings/apply';
import { parseSettings, type SettingsFile } from '../src/settings/schema';

function local(): CurrentSettings {
	return {
		core: {
			dialect: 'American',
			lint_config: { SpellCheck: true, LongSentences: null, RepeatedWords: false },
			user_dictionary: ['Harper', 'localonly', 'Harper'],
			ignored_lints: '{"context_hashes":[18446744073709551615,1]}',
		},
		extension: {
			domain_status: { 'github.com': true, 'local.example': false },
			default_enabled: false,
			delay: 0,
			activation_key: 'off',
			hotkey: { modifiers: ['Ctrl'], key: 'e' },
			isolate_english: false,
		},
	};
}

function incoming(): SettingsFile {
	return {
		schema_version: 1,
		exported_at: '2026-10-09T10:00:00.000Z',
		source_app: 'harper-firefox',
		harper_version: '1.0.0',
		core: {
			dialect: 'British',
			lint_config: { SpellCheck: false, LongSentences: true },
			user_dictionary: ['Harper', 'fromfile'],
			ignored_lints: '{"context_hashes":[2,18446744073709551615]}',
		},
		extension: {
			domain_status: { 'github.com': false, 'remote.example': true },
			default_enabled: true,
			delay: 500,
			activation_key: 'control',
			hotkey: { modifiers: ['Alt'], key: 'k' },
			isolate_english: true,
		},
	};
}

test.describe('settings import/export logic', () => {
	test('export round-trips through the parser', () => {
		const exported = buildExport(local(), {
			sourceApp: 'harper-chrome',
			harperVersion: '1.0.0',
			now: new Date('2026-10-09T10:00:00.000Z'),
		});
		const reparsed = parseSettings(JSON.stringify(exported));

		expect(reparsed).toEqual(exported);
		expect(reparsed.core.user_dictionary).toEqual(['Harper', 'localonly']);
		expect(reparsed.core.lint_config.LongSentences).toBeNull();
	});

	test('importing an export of the same profile changes nothing', () => {
		const current = local();
		const exported = buildExport(current, { sourceApp: 'harper-chrome', harperVersion: '1' });

		for (const mode of ['merge', 'replace'] as const) {
			const summary = summarizeImport(current, exported, { mode, includeExtension: true });
			expect(summary).toEqual({
				dialectChanged: false,
				wordsAdded: 0,
				wordsRemoved: 0,
				ruleChanges: 0,
				ignoredLintsAdded: 0,
				domainChanges: 0,
				extensionIncluded: true,
			});
		}
	});

	test('merge combines collections and imported keys win', () => {
		const next = computeImportedSettings(local(), incoming(), {
			mode: 'merge',
			includeExtension: true,
		});

		expect(next.core.dialect).toBe('British');
		expect(next.core.user_dictionary).toEqual(['Harper', 'localonly', 'fromfile']);
		expect(next.core.lint_config).toEqual({
			SpellCheck: false,
			LongSentences: true,
			RepeatedWords: false,
		});
		expect(extractIgnoredLintHashes(next.core.ignored_lints).sort()).toEqual(
			['1', '18446744073709551615', '2'].sort(),
		);
		expect(next.extension.domain_status).toEqual({
			'github.com': false,
			'local.example': false,
			'remote.example': true,
		});
	});

	test('replace overwrites collections', () => {
		const next = computeImportedSettings(local(), incoming(), {
			mode: 'replace',
			includeExtension: true,
		});

		expect(next.core.user_dictionary).toEqual(['Harper', 'fromfile']);
		expect(next.core.lint_config).toEqual({ SpellCheck: false, LongSentences: true });
		expect(extractIgnoredLintHashes(next.core.ignored_lints)).toEqual([
			'2',
			'18446744073709551615',
		]);
		expect(next.extension.domain_status).toEqual({
			'github.com': false,
			'remote.example': true,
		});
	});

	test('scalars take the imported value in both modes', () => {
		for (const mode of ['merge', 'replace'] as const) {
			const next = computeImportedSettings(local(), incoming(), { mode, includeExtension: true });

			expect(next.core.dialect).toBe('British');
			expect(next.extension.delay).toBe(500);
			expect(next.extension.activation_key).toBe('control');
			expect(next.extension.hotkey).toEqual({ modifiers: ['Alt'], key: 'k' });
			expect(next.extension.default_enabled).toBe(true);
			expect(next.extension.isolate_english).toBe(true);
		}
	});

	test('extension block is left alone unless requested', () => {
		const current = local();
		const plan = planImport(current, incoming(), { mode: 'replace', includeExtension: false });

		expect(Object.keys(plan.set).sort()).toEqual(['ignoredLints', 'lintConfig', 'userDictionary']);
		expect(plan.remove).toEqual([]);
		expect(
			computeImportedSettings(current, incoming(), { mode: 'replace', includeExtension: false })
				.extension,
		).toEqual(current.extension);
	});

	test('a file without an extension block never touches extension settings', () => {
		const file = incoming();
		delete file.extension;
		const plan = planImport(local(), file, { mode: 'replace', includeExtension: true });

		expect(Object.keys(plan.set).sort()).toEqual(['ignoredLints', 'lintConfig', 'userDictionary']);
	});

	test('plan writes storage keys in the shape the extension expects', () => {
		const plan = planImport(local(), incoming(), { mode: 'merge', includeExtension: true });

		expect(plan.dialect).toBe('British');
		expect(typeof plan.set.lintConfig).toBe('string');
		expect(JSON.parse(plan.set.lintConfig as string).SpellCheck).toBe(false);
		expect(plan.set.ignoredLints).toBe('{"context_hashes":[18446744073709551615,1,2]}');
		expect(plan.set['domainStatus remote.example']).toBe(true);
		expect(plan.set.activationKey).toBe('control');
		expect(plan.remove).toEqual([]);
	});

	test('replace removes local domains missing from the file', () => {
		const plan = planImport(local(), incoming(), { mode: 'replace', includeExtension: true });

		expect(plan.remove).toEqual(['domainStatus local.example']);
	});

	test('summary counts what will change', () => {
		const summary = summarizeImport(local(), incoming(), {
			mode: 'replace',
			includeExtension: true,
		});

		expect(summary).toEqual({
			dialectChanged: true,
			wordsAdded: 1,
			wordsRemoved: 1,
			ruleChanges: 3,
			ignoredLintsAdded: 1,
			domainChanges: 3,
			extensionIncluded: true,
		});
	});

	test('hash extraction keeps full 64-bit precision', () => {
		expect(extractIgnoredLintHashes('{"context_hashes":[18446744073709551615]}')).toEqual([
			'18446744073709551615',
		]);
		expect(extractIgnoredLintHashes('{"context_hashes":[]}')).toEqual([]);
	});
});
