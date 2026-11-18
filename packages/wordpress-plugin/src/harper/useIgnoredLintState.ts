import { useDispatch, useSelect } from '@wordpress/data';
import type { Lint } from 'harper.js';
import { useCallback } from 'react';
import { useLinter } from './LinterProvider';
import type { PreferencesActions, PreferencesSelectors } from './preferencesStoreTypes';

const KEY = 'ignoredLints';

export default function useIgnoredLintState(): [string | undefined, (newState: string) => void] {
	const ignoredLintState = useSelect(
		(select) => (select('core/preferences') as PreferencesSelectors).get('harper-wp', KEY),
		[],
	);

	const { set } = useDispatch('core/preferences') as PreferencesActions;

	const updateState = useCallback((newValue: string) => set('harper-wp', KEY, newValue), [set]);

	return [ignoredLintState, updateState];
}

/** Ignore a lint using the exact source text that produced its spans and context. */
export function useIgnoreLint(): (source: string, lint: Lint) => Promise<void> {
	const linter = useLinter();
	const [ignoredLintState, setIgnoredLintState] = useIgnoredLintState();

	return async (source, lint) => {
		await linter.clearIgnoredLints();

		if (ignoredLintState) {
			await linter.importIgnoredLints(ignoredLintState);
		}

		await linter.ignoreLint(source, lint);
		setIgnoredLintState(await linter.exportIgnoredLints());
	};
}
