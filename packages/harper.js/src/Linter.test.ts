import { expect, test } from 'vitest';
import { binary } from './binary';
import LocalLinter from './LocalLinter';
import WorkerLinter from './WorkerLinter';

const linters = {
	WorkerLinter: WorkerLinter,
	LocalLinter: LocalLinter,
};

for (const [linterName, Linter] of Object.entries(linters)) {
	test(`${linterName} detects repeated words`, async () => {
		const linter = new Linter({ binary });

		const lints = await linter.lint('The the problem is...');

		expect(lints.length).toBe(1);
	});

	test(`${linterName} emits organized lints the same as it emits normal lints`, async () => {
		const linter = new Linter({ binary });
		const source = 'The the problem is...';

		const lints = await linter.lint(source);
		expect(lints.length).toBeGreaterThan(0);

		const organized = await linter.organizedLints(source);
		const normal = await linter.lint(source);

		const flattened = [];
		for (const [_, value] of Object.entries(organized)) {
			flattened.push(...value);
		}

		expect(flattened.length).toBe(1);
		expect(flattened.length).toBe(normal.length);

		const item = flattened[0];
		expect(item.message().length).not.toBe(0);
	});

	test(`${linterName} detects repeated words with multiple synchronous requests`, async () => {
		const linter = new Linter({ binary });

		const promises = [
			linter.lint('The problem is that that...'),
			linter.lint('The problem is...'),
			linter.lint('The the problem is...'),
		];

		const results = await Promise.all(promises);

		expect(results[0].length).toBe(1);
		expect(results[0][0].suggestions().length).toBe(1);
		expect(results[1].length).toBe(0);
		expect(results[2].length).toBe(1);
	});

	test(`${linterName} detects repeated words with concurrent requests`, async () => {
		const linter = new Linter({ binary });

		const promises = [
			linter.lint('The problem is that that...'),
			linter.lint('The problem is...'),
			linter.lint('The the problem is...'),
		];

		const results = await Promise.all(promises);

		expect(results[0].length).toBe(1);
		expect(results[0][0].suggestions().length).toBe(1);
		expect(results[1].length).toBe(0);
		expect(results[2].length).toBe(1);
	});

	test(`${linterName} detects lorem ipsum paragraph as not english`, async () => {
		const linter = new Linter({ binary });

		const result = await linter.isLikelyEnglish(
			'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.',
		);

		expect(result).toBeTypeOf('boolean');
		expect(result).toBe(false);
	});

	test(`${linterName} can run setup without issues`, async () => {
		const linter = new Linter({ binary });

		await linter.setup();
	});

	test(`${linterName} contains configuration option for repetition`, async () => {
		const linter = new Linter({ binary });

		const lintConfig = await linter.getLintConfig();
		expect(lintConfig).toHaveProperty('RepeatedWords');
	});

	test(`${linterName} can set its configuration away and to default`, async () => {
		const linter = new Linter({ binary });

		let lintConfig = await linter.getLintConfig();

		for (const key of Object.keys(lintConfig)) {
			lintConfig[key] = true;
		}

		await linter.setLintConfig(lintConfig);
		lintConfig = await linter.getLintConfig();

		for (const key of Object.keys(lintConfig)) {
			lintConfig[key] = null;
		}

		await linter.setLintConfig(lintConfig);
		lintConfig = await linter.getLintConfig();

		for (const key of Object.keys(lintConfig)) {
			expect(lintConfig[key]).toBe(null);
		}
	});

	test(`${linterName} can both get and set its configuration`, async () => {
		const linter = new Linter({ binary });

		let lintConfig = await linter.getLintConfig();

		for (const key of Object.keys(lintConfig)) {
			lintConfig[key] = true;
		}

		await linter.setLintConfig(lintConfig);
		lintConfig = await linter.getLintConfig();

		for (const key of Object.keys(lintConfig)) {
			expect(lintConfig[key]).toBe(true);
		}
	});

	test(`${linterName} can make things title case`, async () => {
		const linter = new Linter({ binary });

		const titleCase = await linter.toTitleCase('this is a test for making titles');

		expect(titleCase).toBe('This Is a Test for Making Titles');
	});

	test(`${linterName} can get rule descriptions`, async () => {
		const linter = new Linter({ binary });

		const descriptions = await linter.getLintDescriptions();

		expect(descriptions).toBeTypeOf('object');
	});

	test(`${linterName} can get rule descriptions in HTML.`, async () => {
		const linter = new Linter({ binary });

		const descriptions = await linter.getLintDescriptionsHTML();

		expect(descriptions).toBeTypeOf('object');
	});

	test(`${linterName} rule descriptions are not empty`, async () => {
		const linter = new Linter({ binary });

		const descriptions = await linter.getLintDescriptions();

		for (const value of Object.values(descriptions)) {
			expect(value).toBeTypeOf('string');
			expect(value).not.toHaveLength(0);
		}
	});

	test(`${linterName} default lint config has no null values`, async () => {
		const linter = new Linter({ binary });

		const lintConfig = await linter.getDefaultLintConfig();

		for (const value of Object.values(lintConfig)) {
			expect(value).not.toBeNull();
		}
	});

	test(`${linterName} can generate lint context hashes`, async () => {
		const linter = new Linter({ binary });
		const source = 'This is an test.';

		const lints = await linter.lint(source);

		expect(lints.length).toBeGreaterThanOrEqual(1);

		await linter.contextHash(source, lints[0]);
	});

	test(`${linterName} can ignore lints`, async () => {
		const linter = new Linter({ binary });
		const source = 'This is an test.';

		const firstRound = await linter.lint(source);

		expect(firstRound.length).toBeGreaterThanOrEqual(1);

		await linter.ignoreLint(source, firstRound[0]);

		const secondRound = await linter.lint(source);

		expect(secondRound.length).toBeLessThan(firstRound.length);
	});

	test(`${linterName} can ignore lints with hashes`, async () => {
		const linter = new Linter({ binary });
		const source = 'This is an test.';

		const firstRound = await linter.lint(source);

		expect(firstRound.length).toBeGreaterThanOrEqual(1);

		const hash = await linter.contextHash(source, firstRound[0]);
		await linter.ignoreLintHash(hash);

		const secondRound = await linter.lint(source);

		expect(secondRound.length).toBeLessThan(firstRound.length);
	});

	test(`${linterName} can ignore larger lints to reveal smaller ones`, async () => {
		const linter = new Linter({ binary });
		const source = `This is a really long sentensd with some errorz in it, which in an old version of Harper, would get removedd when the bigger "Long Sentences" lint was ignored, that isn't what we woant, so we are writing a test for that exact problem.`;

		const firstRound = await linter.lint(source);

		expect(firstRound.length).toBeGreaterThanOrEqual(1);

		await linter.ignoreLint(source, firstRound[0]);

		const secondRound = await linter.lint(source);

		expect(secondRound.length).toBe(4);
	});

	test(`${linterName} can reimport ignored lints.`, async () => {
		const source = 'This is an test of xporting lints.';

		const firstLinter = new Linter({ binary });

		const firstLints = await firstLinter.lint(source);

		for (const lint of firstLints) {
			await firstLinter.ignoreLint(source, lint);
		}

		const exported = await firstLinter.exportIgnoredLints();

		/// Create a new instance and reimport the lints.
		const secondLinter = new Linter({ binary });
		await secondLinter.importIgnoredLints(exported);

		const secondLints = await secondLinter.lint(source);

		expect(firstLints.length).toBeGreaterThan(secondLints.length);
		expect(secondLints.length).toBe(0);
	});

	test(`${linterName} can add words to the dictionary`, async () => {
		const source = 'asdf is not a word';

		const linter = new Linter({ binary });
		let lints = await linter.lint(source);

		expect(lints).toHaveLength(1);

		await linter.importWords(['asdf']);
		lints = await linter.lint(source);

		expect(lints).toHaveLength(0);
	});

	test(`${linterName} allows correct capitalization of "United States"`, async () => {
		const linter = new Linter({ binary });
		const lints = await linter.lint('The United States is a big country.');

		expect(lints).toHaveLength(0);
	});

	test(`${linterName} can summarize simple stat records`, async () => {
		const linter = new Linter({ binary });
		linter.setup();

		const source = 'This is an test.';

		const lints = await linter.lint(source);

		const lint = lints[0];

		expect(lint).not.toBeNull();

		const sug = lint.suggestions()[0];

		expect(sug).not.toBeNull();

		const applied = await linter.applySuggestion(source, lint, sug);

		expect(applied).toBe('This is a test.');

		const summary = await linter.summarizeStats();
		expect(summary).toBeTypeOf('object');
	});

	test(`${linterName} can save and restore stat records`, async () => {
		const linter = new Linter({ binary });
		linter.setup();

		const source = 'This is an test.';

		const lints = await linter.lint(source);

		const lint = lints[0];

		expect(lint).not.toBeNull();

		const sug = lint.suggestions()[0];

		expect(sug).not.toBeNull();

		const applied = await linter.applySuggestion(source, lint, sug);

		expect(applied).toBe('This is a test.');

		const stats = await linter.generateStatsFile();

		const newLinter = new Linter({ binary });
		await newLinter.importStatsFile(stats);
	});
}

test('Linters have the same config format', async () => {
	const configs = [];

	for (const Linter of Object.values(linters)) {
		const linter = new Linter({ binary });

		configs.push(await linter.getLintConfig());
	}

	for (const config of configs) {
		expect(config).toEqual(configs[0]);
		expect(config).toBeTypeOf('object');
	}
});

test('Linters have the same JSON config format', async () => {
	const configs = [];

	for (const Linter of Object.values(linters)) {
		const linter = new Linter({ binary });

		configs.push(await linter.getLintConfigAsJSON());
	}

	for (const config of configs) {
		expect(config).toEqual(configs[0]);
		expect(config).toBeTypeOf('string');
	}
});
