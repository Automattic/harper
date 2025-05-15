import { shuffle } from 'lodash-es';
import { expect, test } from 'vitest';
import Logic from './Logic';

/** Create an instance of the test class that doesn't use external state. */
function createEphemeralLogic(): Logic {
	return new Logic(
		(_) => Promise.resolve(),
		() => {},
	);
}

test('Toggling linting should change extension array.', () => {
	const logic = createEphemeralLogic();

	const editorExtensions = logic.getCMEditorExtensions();
	logic.enableEditorLinter();

	expect(editorExtensions.length).toBe(1);

	logic.disableEditorLinter();

	expect(editorExtensions.length).toBe(0);
});

test('Passing default settings back in should have a null net change.', async () => {
	const logic = createEphemeralLogic();

	const initialSettings = await logic.getSettings();
	await logic.initializeFromSettings(initialSettings);
	const reinitSettings = await logic.getSettings();

	expect(reinitSettings).toStrictEqual(initialSettings);
});

test('Default settings should have null linter configs', async () => {
	const logic = createEphemeralLogic();

	const defaultSettings = await logic.getSettings();

	const linterKeys = Object.keys(defaultSettings.lintSettings);

	expect(linterKeys.length).toBeGreaterThan(0);

	for (const key of linterKeys) {
		const setting = defaultSettings.lintSettings[key];
		expect(setting).toBeNull();
	}
});

test('Lint keys are not undefined', async () => {
	const logic = createEphemeralLogic();

	const defaultSettings = await logic.getSettings();

	expect(defaultSettings.lintSettings.ThisKeyDoesNotExist).toBeUndefined();
	expect(defaultSettings.lintSettings.RepeatedWords).toBeNull();
});

test('Lint keys can be enabled, then set to default.', async () => {
	const logic = createEphemeralLogic();

	let settings = await logic.getSettings();

	settings.lintSettings.RepeatedWords = true;
	await logic.initializeFromSettings(settings);
	settings = await logic.getSettings();
	expect(settings.lintSettings.RepeatedWords).toBe(true);

	settings.lintSettings.RepeatedWords = null;
	await logic.initializeFromSettings(settings);
	settings = await logic.getSettings();
	expect(settings.lintSettings.RepeatedWords).toBe(null);
});

test('Lint settings and descriptions have the same keys', async () => {
	const logic = createEphemeralLogic();

	const settings = await logic.getSettings();
	const descriptions = await logic.getDescriptions();

	expect(Object.keys(descriptions).sort()).toStrictEqual(Object.keys(settings.lintSettings).sort());
});

test('Can be initialized with incomplete lint settings and retain default state.', async () => {
	const logic = createEphemeralLogic();

	// Get the default settings
	const defaultSettings = await logic.getSettings();

	// Pick just a few lint settings to keep.
	const numberToKeep = 5;
	const reducedLintSettings = Object.fromEntries(
		shuffle(Object.entries(defaultSettings.lintSettings)).slice(0, numberToKeep),
	);
	expect(Object.keys(reducedLintSettings).length).toBe(numberToKeep);

	await logic.initializeFromSettings({ ...defaultSettings, lintSettings: reducedLintSettings });

	expect(await logic.getSettings()).toStrictEqual(defaultSettings);
});
