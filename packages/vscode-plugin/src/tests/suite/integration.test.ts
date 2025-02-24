import type { Extension } from 'vscode';

import { commands, ConfigurationTarget, Uri, workspace } from 'vscode';

import {
	activateHarper,
	compareActualVsExpectedDiagnostics,
	createExpectedDiagnostics,
	createRange,
	getActualDiagnostics,
	openFile,
	sleep
} from './helper';

describe('Integration >', () => {
	let harper: Extension<void>;
	let markdownUri: Uri;

	beforeAll(async () => {
		harper = await activateHarper();
		// Open test file so diagnostics can occur
		markdownUri = await openFile('integration.md');
		// Wait for `harper-ls` to start
		await sleep(500);
	});

	it('runs', () => {
		expect(harper.isActive).toBe(true);
	});

	it('gives correct diagnostics', () => {
		compareActualVsExpectedDiagnostics(
			getActualDiagnostics(markdownUri),
			createExpectedDiagnostics(
				{
					message: 'Did you mean to repeat this word?',
					range: createRange(2, 39, 2, 48)
				},
				{
					message: 'Did you mean to spell “errorz” this way?',
					range: createRange(2, 26, 2, 32)
				}
			)
		);
	});

	it('updates diagnostics on configuration change', async () => {
		const config = workspace.getConfiguration('harper.linters');
		await config.update('RepeatedWords', false, ConfigurationTarget.Workspace);
		// Wait for `harper-ls` to update diagnostics
		await sleep(300);

		compareActualVsExpectedDiagnostics(
			getActualDiagnostics(markdownUri),
			createExpectedDiagnostics({
				message: 'Did you mean to spell “errorz” this way?',
				range: createRange(2, 26, 2, 32)
			})
		);

		// Set config back to default value
		await config.update('RepeatedWords', true, ConfigurationTarget.Workspace);
	});

	it('updates diagnostics when files are deleted', async () => {
		const markdownContent = await workspace.fs.readFile(markdownUri);

		// Delete file through VSCode
		await commands.executeCommand('workbench.files.action.showActiveFileInExplorer');
		await commands.executeCommand('deleteFile');
		// Wait for `harper-ls` to update diagnostics
		await sleep(450);

		compareActualVsExpectedDiagnostics(
			getActualDiagnostics(markdownUri),
			createExpectedDiagnostics()
		);

		// Restore and reopen deleted file
		await workspace.fs.writeFile(markdownUri, markdownContent);
		await openFile('integration.md');
		// Wait for `harper-ls` to update diagnostics
		await sleep(75);

		// Delete file directly
		await workspace.fs.delete(markdownUri);
		// Wait for `harper-ls` to update diagnostics
		await sleep(450);

		compareActualVsExpectedDiagnostics(
			getActualDiagnostics(markdownUri),
			createExpectedDiagnostics()
		);

		// Restore and reopen deleted file
		await workspace.fs.writeFile(markdownUri, markdownContent);
		await openFile('integration.md');
	});
});
