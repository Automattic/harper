import type { Diagnostic, Extension } from 'vscode';

import {
	DiagnosticSeverity,
	Position,
	Range,
	Uri,
	extensions,
	languages,
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

export async function openFile(...pathSegments: string[]): Promise<Uri> {
	const uri = Uri.joinPath(Uri.file(workspace.workspaceFolders![0].uri.path), ...pathSegments);
	await window.showTextDocument(await workspace.openTextDocument(uri));
	return uri;
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

export function getActualDiagnostics(resource: Uri): Diagnostic[] {
	return languages.getDiagnostics(resource).filter((d) => d.source === 'Harper');
}

export function createExpectedDiagnostics(
	...data: { message: string; range: Range }[]
): Diagnostic[] {
	return data.map((d) => ({ ...d, source: 'Harper', severity: DiagnosticSeverity.Information }));
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

// The numbers used in these functions are what works when running tests in GitHub CI.
export async function waitForHarperToActivate() {
	await sleep(500);
}
export async function waitForUpdatesFromOpenedFile() {
	await sleep(75);
}
export async function waitForUpdatesFromConfigChange() {
	await sleep(300);
}
export async function waitForUpdatesFromDeletedFile() {
	await sleep(450);
}
function sleep(duration: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, duration));
}
