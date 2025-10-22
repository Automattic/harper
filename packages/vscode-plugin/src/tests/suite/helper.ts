import type { Diagnostic, Extension } from 'vscode';

import {
	DiagnosticSeverity,
	extensions,
	languages,
	Position,
	Range,
	Uri,
	window,
	workspace,
} from 'vscode';

export async function closeAll(): Promise<void> {
	for (const tabGroup of window.tabGroups.all) {
		await window.tabGroups.close(tabGroup);
	}
}

export async function activateHarper(): Promise<Extension<void>> {
	const harper = extensions.getExtension('elijah-potter.harper')!;

	if (!harper.isActive) {
		await harper.activate();
	}

	return harper;
}

export function getUri(...pathSegments: string[]): Uri {
	return Uri.joinPath(Uri.file(workspace.workspaceFolders![0].uri.path), ...pathSegments);
}

export async function openUri(uri: Uri): Promise<void> {
	await window.showTextDocument(await workspace.openTextDocument(uri));
}

export async function openUntitled(text: string): Promise<Uri> {
	const document = await workspace.openTextDocument();
	const editor = await window.showTextDocument(document);
	await editor.edit((editBuilder) => editBuilder.insert(new Position(0, 0), text));
	return document.uri;
}

export async function setTextDocumentLanguage(uri: Uri, languageId: string): Promise<void> {
	const document = await workspace.openTextDocument(uri);
	languages.setTextDocumentLanguage(document, languageId);
}

export function createExpectedDiagnostics(
	...data: { message: string; range: Range; source: string }[]
): Diagnostic[] {
	return data.map((d) => ({ ...d, severity: DiagnosticSeverity.Information }));
}

export function compareActualVsExpectedDiagnostics(
	actual: Diagnostic[],
	expected: Diagnostic[],
): void {
	if (actual.length !== expected.length) {
		throw new Error(`Expected ${expected.length} diagnostics, got ${actual.length}.`);
	}

	for (let i = 0; i < actual.length; i++) {
		expect(actual[i].source).toBe(expected[i].source);
		expect(actual[i].message).toBe(expected[i].message);
		expect(actual[i].severity).toBe(expected[i].severity);
		expect(actual[i].range).toEqual(expected[i].range);
	}
}

export function createRange(
	startRow: number,
	startColumn: number,
	endRow: number,
	endColumn: number,
): Range {
	return new Range(new Position(startRow, startColumn), new Position(endRow, endColumn));
}

function getActualDiagnostics(resource: Uri): Diagnostic[] {
	return languages.getDiagnostics(resource).filter((d) => d.source?.includes('Harper'));
}

/** Note that this function times out if there is no change detected. */
export function waitForDiagnosticsChange(
	uri: Uri,
	func?: () => Promise<void>,
): Promise<Diagnostic[]> {
	return new Promise((resolve, reject) => {
		const before = func ? getActualDiagnostics(uri) : [];

		(func || (async () => {}))().then(() => {
			const delay = 10;
			const limit = 10;
			let counter = 0;

			const tryCompare = () => {
				const after = getActualDiagnostics(uri);
				try {
					compareActualVsExpectedDiagnostics(before, after);

					// after didn't change, try again
					counter = 0;
					tryAgain();
				} catch (e) {
					// after did change, try until stabilized
					counter++;
					if (counter < limit) {
						tryAgain();
					} else {
						clearTimeout(rejectTimer);
						resolve(after);
					}
				}
			};
			let tryTimer = setTimeout(tryCompare);
			const tryAgain = () => {
				tryTimer = setTimeout(tryCompare, delay);
			};

			const rejectTimer = setTimeout(() => {
				clearTimeout(tryTimer);
				reject('No change of diagnostics detected.');
			}, 4000);
		});
	});
}
