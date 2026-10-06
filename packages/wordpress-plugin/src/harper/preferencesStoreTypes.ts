import type { Dialect, LintConfig } from 'harper.js';

type HarperPreferences = {
	dialect: Dialect;
	lintConfig: LintConfig;
	ignoredLints: string;
	personalDictionary: string[];
};

/** Selector types for Harper's keys in Gutenberg's legacy named preferences store. */
export type PreferencesSelectors = {
	get<K extends keyof HarperPreferences>(
		scope: 'harper-wp',
		key: K,
	): HarperPreferences[K] | undefined;
};

/** Action types for Harper's keys in Gutenberg's legacy named preferences store. */
export type PreferencesActions = {
	set<K extends keyof HarperPreferences>(
		scope: 'harper-wp',
		key: K,
		value: HarperPreferences[K],
	): void;
};
