import type { ExtensionContext } from 'vscode';
import { commands, StatusBarAlignment, type StatusBarItem, Uri, window, workspace } from 'vscode';
import type { Executable, LanguageClientOptions } from 'vscode-languageclient/node';
import { LanguageClient, ResponseError, TransportKind } from 'vscode-languageclient/node';

// There's no publicly available extension manifest type except for the internal one from VS Code's
// codebase. So, we declare our own with only the fields we need and have. See:
// https://stackoverflow.com/a/78536803
type ExtensionManifest = {
	activationEvents: string[];
	contributes: { configuration: { properties: { [key: string]: object } } };
};

let client: LanguageClient | undefined;
const serverOptions: Executable = { command: '', transport: TransportKind.stdio };
const clientOptions: LanguageClientOptions = {
	middleware: {
		workspace: {
			async configuration(params, token, next) {
				const response = await next(params, token);

				if (response instanceof ResponseError) {
					return response;
				}

				return [{ 'harper-ls': response[0].harper }];
			},
		},
		executeCommand(command, args, next) {
			if (
				[
					'HarperAddToUserDict',
					'HarperAddToWSDict',
					'HarperAddToFileDict',
					'HarperIgnoreLint',
				].includes(command) &&
				args.find((a) => typeof a === 'string' && a.startsWith('untitled:'))
			) {
				window
					.showInformationMessage('Save the file to execute this command.', 'Save File', 'Dismiss')
					.then((selected) => {
						if (selected === 'Save File') {
							commands.executeCommand('workbench.action.files.save');
						}
					});
				return;
			}

			next(command, args);
		},
	},
};

let dialectStatusBarItem: StatusBarItem | undefined;

export async function activate(context: ExtensionContext): Promise<void> {
	serverOptions.command = getExecutablePath(context);

	let manifest: ExtensionManifest;
	try {
		manifest = JSON.parse(
			(await workspace.fs.readFile(Uri.joinPath(context.extensionUri, 'package.json'))).toString(),
		);
	} catch (error) {
		showError('Failed to read manifest file', error);
		return;
	}

	clientOptions.documentSelector = manifest.activationEvents
		.filter((e) => e.startsWith('onLanguage:'))
		.flatMap((e) => {
			const language = e.split(':')[1];
			return [
				{ language, scheme: 'file' },
				{ language, scheme: 'untitled' },
			];
		});

	clientOptions.outputChannel = window.createOutputChannel('Harper');
	context.subscriptions.push(clientOptions.outputChannel);

	const configs = Object.keys(manifest.contributes.configuration.properties);
	context.subscriptions.push(
		workspace.onDidChangeConfiguration(async (event) => {
			if (event.affectsConfiguration('harper.path')) {
				serverOptions.command = getExecutablePath(context);
				await startLanguageServer();
				return;
			}

			if (configs.find((c) => event.affectsConfiguration(c))) {
				await client?.sendNotification('workspace/didChangeConfiguration', {
					settings: { 'harper-ls': workspace.getConfiguration('harper') },
				});
			}
		}),
	);

	context.subscriptions.push(
		commands.registerCommand('harper.languageserver.restart', startLanguageServer),
	);

	await startLanguageServer();

	// VS Code:
	// <= 100 is between Copilot and Notifications.
	// 101..102 is between the magnifying glass and encoding
	// >= 103 is left of the magnifying glass
	// Windsurf:
	// 100 is just to the right of programming language - perfect!
	// 101 is left of line/column
	dialectStatusBarItem = window.createStatusBarItem(StatusBarAlignment.Right, 100);
	dialectStatusBarItem.tooltip = 'Harper English dialect';
	context.subscriptions.push(dialectStatusBarItem);

	context.subscriptions.push(
		workspace.onDidChangeConfiguration(async (event) => {
			if (event.affectsConfiguration('harper.dialect')) {
				updateDialectStatusBar();
			}
		}),
	);

	updateDialectStatusBar();
}

function getExecutablePath(context: ExtensionContext): string {
	const path = workspace.getConfiguration('harper').get<string>('path', '');

	if (path !== '') {
		return path;
	}

	return Uri.joinPath(
		context.extensionUri,
		'bin',
		`harper-ls${process.platform === 'win32' ? '.exe' : ''}`,
	).fsPath;
}

async function startLanguageServer(): Promise<void> {
	if (client?.needsStop()) {
		if (client.diagnostics) {
			client.diagnostics.clear();
		}

		try {
			await client.stop(2000);
		} catch (error) {
			showError('Failed to stop harper-ls', error);
			return;
		}
	}

	try {
		client = new LanguageClient('harper', 'Harper', serverOptions, clientOptions);
		await client.start();
	} catch (error) {
		showError('Failed to start harper-ls', error);
		client = undefined;
	}
}

function showError(message: string, error: Error | unknown): void {
	let info = '';
	if (error instanceof Error) {
		info = error.stack ? error.stack : error.message;
	}

	window.showErrorMessage(message, 'Show Info', 'Dismiss').then((selected) => {
		if (selected === 'Show Info') {
			clientOptions.outputChannel?.appendLine('---');
			clientOptions.outputChannel?.appendLine(message);
			clientOptions.outputChannel?.appendLine(info);
			clientOptions.outputChannel?.appendLine(
				'If the issue persists, please report at https://github.com/automattic/harper/issues',
			);
			clientOptions.outputChannel?.appendLine('---');
			clientOptions.outputChannel?.show();
		}
	});
}

function updateDialectStatusBar(): void {
	if (!dialectStatusBarItem) return;

	const dialect = workspace.getConfiguration('harper').get<string>('dialect', '');
	if (dialect === '') return;

	const flagAndCode = getFlagAndCode(dialect);
	if (!flagAndCode) return;

	dialectStatusBarItem.text = `$(harper-logo) ${flagAndCode.join(' ')}`;
	dialectStatusBarItem.show();
	console.log(`** dialect set to ${dialect} **`, dialect);
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}

	return client.stop();
}

function getFlagAndCode(dialect: string): string[] | undefined {
	return {
		American: ['🇺🇸', 'US'],
		Australian: ['🇦🇺', 'AU'],
		British: ['🇬🇧', 'GB'],
		Canadian: ['🇨🇦', 'CA'],
	}[dialect];
}
