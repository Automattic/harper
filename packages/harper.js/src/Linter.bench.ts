import { bench } from 'vitest';
import { binary } from './binary';
import LocalLinter from './LocalLinter';
import WorkerLinter from './WorkerLinter';

const linters = {
	WorkerLinter: WorkerLinter,
	LocalLinter: LocalLinter,
};

for (const [linterName, Linter] of Object.entries(linters)) {
	const linter = new Linter({ binary });

	// Prime caches
	linter.setup();

	const defaultConfig = await linter.getDefaultLintConfig();
	const emptyIgnoreState = await linter.exportIgnoredLints();

	bench(`${linterName} set lint configuration`, async () => {
		await linter.setLintConfig(defaultConfig);
	});

	bench(`${linterName} get lint configuration`, async () => {
		await linter.getLintConfig();
	});

	bench(`${linterName} reset ignore state`, async () => {
		await linter.clearIgnoredLints();
		await linter.importIgnoredLints(emptyIgnoreState);
	});
}
