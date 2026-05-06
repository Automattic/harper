import type { Linter } from 'harper.js';

export async function createEditorLinter(): Promise<Linter> {
	const [{ WorkerLinter }, { slimBinary }] = await Promise.all([
		import('harper.js'),
		import('harper.js/slimBinary'),
	]);

	const linter = new WorkerLinter({ binary: slimBinary });
	await linter.setup();
	return linter;
}
