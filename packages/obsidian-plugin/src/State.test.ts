import { shuffle } from 'lodash-es';
import { expect, test } from 'vitest';
import State from './State';

/** Create an instance of the test class that doesn't use external persistence. */
function createEphemeralState(): State {
	return new State(
		(_) => Promise.resolve(),
		() => {},
	);
}

test('Toggling linting should change extension array.', () => {
	const state = createEphemeralState();

	const editorExtensions = state.getCMEditorExtensions();
	state.enableEditorLinter();

	expect(editorExtensions.length).toBe(1);

	state.disableEditorLinter();

	expect(editorExtensions.length).toBe(0);
});

test('Passing default settings back in should have a null net change.', async () => {
	const state = createEphemeralState();

	const initialSettings = await state.getSettings();
	await state.initializeFromSettings(initialSettings);
	const reinitSettings = await state.getSettings();

	expect(reinitSettings).toStrictEqual(initialSettings);
});

test('Default settings should have null linter configs', async () => {
	const state = createEphemeralState();

	const defaultSettings = await state.getSettings();

	const linterKeys = Object.keys(defaultSettings.lintSettings);

	expect(linterKeys.length).toBeGreaterThan(0);

	for (const key of linterKeys) {
		const setting = defaultSettings.lintSettings[key];
		expect(setting).toBeNull();
	}
});

test('Lint keys are not undefined', async () => {
	const state = createEphemeralState();

	const defaultSettings = await state.getSettings();

	expect(defaultSettings.lintSettings.ThisKeyDoesNotExist).toBeUndefined();
	expect(defaultSettings.lintSettings.RepeatedWords).toBeNull();
});

test('Lint keys can be enabled, then set to default.', async () => {
	const state = createEphemeralState();

	let settings = await state.getSettings();

	settings.lintSettings.RepeatedWords = true;
	await state.initializeFromSettings(settings);
	settings = await state.getSettings();
	expect(settings.lintSettings.RepeatedWords).toBe(true);

	settings.lintSettings.RepeatedWords = null;
	await state.initializeFromSettings(settings);
	settings = await state.getSettings();
	expect(settings.lintSettings.RepeatedWords).toBe(null);
});

test('Lint settings and descriptions have the same keys', async () => {
	const state = createEphemeralState();

	const settings = await state.getSettings();
	const descriptions = await state.getDescriptionHTML();

	expect(Object.keys(descriptions).sort()).toStrictEqual(Object.keys(settings.lintSettings).sort());
});

test('Can be initialized with incomplete lint settings and retain default state.', async () => {
	const state = createEphemeralState();

	// Get the default settings
	const defaultSettings = await state.getSettings();

	// Pick just a few lint settings to keep.
	const numberToKeep = 5;
	const reducedLintSettings = Object.fromEntries(
		shuffle(Object.entries(defaultSettings.lintSettings)).slice(0, numberToKeep),
	);
	expect(Object.keys(reducedLintSettings).length).toBe(numberToKeep);

	await state.initializeFromSettings({ ...defaultSettings, lintSettings: reducedLintSettings });

	expect(await state.getSettings()).toStrictEqual(defaultSettings);
});
