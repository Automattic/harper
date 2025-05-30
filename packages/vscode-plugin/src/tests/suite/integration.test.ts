import type { Extension, Uri } from 'vscode';

import { ConfigurationTarget, commands, workspace } from 'vscode';

import {
	activateHarper,
	closeAll,
	compareActualVsExpectedDiagnostics,
	createExpectedDiagnostics,
	createRange,
	getUri,
	openUntitled,
	openUri,
	setTextDocumentLanguage,
	waitForDiagnosticsChange,
} from './helper';

describe('Integration >', () => {
	let harper: Extension<void>;
	let markdownUri: Uri;

	beforeAll(async () => {
		await closeAll();
		harper = await activateHarper();
		markdownUri = getUri('integration.md');
		await openUri(markdownUri);
	});

	it('runs', () => {
		expect(harper.isActive).toBe(true);
	});

	it('gives correct diagnostics for files', async () => {
		compareActualVsExpectedDiagnostics(
			await waitForDiagnosticsChange(markdownUri),
			createExpectedDiagnostics(
				{
					message: 'Did you mean to repeat this word?',
					range: createRange(2, 39, 2, 48),
				},
				{
					message: 'Did you mean to spell `errorz` this way?',
					range: createRange(2, 26, 2, 32),
				},
				{
					message: 'Did you mean `realize`?',
					range: createRange(4, 26, 4, 33),
				},
			),
		);
	});

	it('gives correct diagnostics for untitled', async () => {
		const untitledUri = await openUntitled('Errorz');

		compareActualVsExpectedDiagnostics(
			await waitForDiagnosticsChange(untitledUri),
			createExpectedDiagnostics({
				message: 'Did you mean to spell `Errorz` this way?',
				range: createRange(0, 0, 0, 6),
			}),
		);
	});

	it('gives correct diagnostics when language is changed', async () => {
		const untitledUri = await openUntitled('Errorz # Errorz');

		compareActualVsExpectedDiagnostics(
			await waitForDiagnosticsChange(
				untitledUri,
				async () => await setTextDocumentLanguage(untitledUri, 'plaintext'),
			),
			createExpectedDiagnostics(
				{
					message: 'Did you mean to spell `Errorz` this way?',
					range: createRange(0, 0, 0, 6),
				},
				{
					message: 'Did you mean to spell `Errorz` this way?',
					range: createRange(0, 9, 0, 15),
				},
			),
		);

		compareActualVsExpectedDiagnostics(
			await waitForDiagnosticsChange(
				untitledUri,
				async () => await setTextDocumentLanguage(untitledUri, 'shellscript'),
			),
			createExpectedDiagnostics({
				message: 'Did you mean to spell `Errorz` this way?',
				range: createRange(0, 9, 0, 15),
			}),
		);
	});

	it('updates diagnostics on configuration change', async () => {
		const config = workspace.getConfiguration('harper.linters');

		compareActualVsExpectedDiagnostics(
			await waitForDiagnosticsChange(
				markdownUri,
				async () => await config.update('RepeatedWords', false, ConfigurationTarget.Workspace),
			),
			createExpectedDiagnostics(
				{
					message: 'Did you mean to spell `errorz` this way?',
					range: createRange(2, 26, 2, 32),
				},
				{
					message: 'Did you mean `realize`?',
					range: createRange(4, 26, 4, 33),
				},
			),
		);

		// Set config back to default value
		await waitForDiagnosticsChange(
			markdownUri,
			async () => await config.update('RepeatedWords', true, ConfigurationTarget.Workspace),
		);
	});

	it('accepts British spellings when dialect is set to British', async () => {
		const config = workspace.getConfiguration('harper');

		compareActualVsExpectedDiagnostics(
			await waitForDiagnosticsChange(
				markdownUri,
				async () => await config.update('dialect', 'British', ConfigurationTarget.Workspace),
			),
			createExpectedDiagnostics(
				{
					message: 'Did you mean to repeat this word?',
					range: createRange(2, 39, 2, 48),
				},
				{
					message: 'Did you mean to spell `errorz` this way?',
					range: createRange(2, 26, 2, 32),
				},
			),
		);

		// Set config back to default value
		await waitForDiagnosticsChange(
			markdownUri,
			async () => await config.update('dialect', 'American', ConfigurationTarget.Workspace),
		);
	});

	it('updates diagnostics when files are deleted', async () => {
		const markdownContent = await workspace.fs.readFile(markdownUri);

		// Delete file through VS Code
		await commands.executeCommand('workbench.files.action.showActiveFileInExplorer');

		compareActualVsExpectedDiagnostics(
			await waitForDiagnosticsChange(
				markdownUri,
				async () => await commands.executeCommand('deleteFile'),
			),
			createExpectedDiagnostics(),
		);

		// Restore and reopen deleted file
		await workspace.fs.writeFile(markdownUri, markdownContent);
		await waitForDiagnosticsChange(markdownUri, async () => await openUri(markdownUri));

		// Delete file directly
		compareActualVsExpectedDiagnostics(
			await waitForDiagnosticsChange(
				markdownUri,
				async () => await workspace.fs.delete(markdownUri),
			),
			createExpectedDiagnostics(),
		);

		// Restore and reopen deleted file
		await workspace.fs.writeFile(markdownUri, markdownContent);
	});
});
