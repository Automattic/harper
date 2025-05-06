import { useDispatch, useSelect } from '@wordpress/data';
import type { Lint } from 'harper.js';
import { useCallback } from 'react';
import { useLinter } from './LinterProvider';

const KEY = 'ignoredLints';

export default function useIgnoredLintState(): [string | undefined, (newState: string) => void] {
	const ignoredLintState = useSelect(
		(select) => select('core/preferences').get('harper-wp', KEY),
		[],
	);

	const { set } = useDispatch('core/preferences');

	const updateState = useCallback((newValue: string) => set('harper-wp', KEY, newValue), [set]);

	return [ignoredLintState, updateState];
}

/** Get a callback that adds a lint to the global ignored lint state. */
export function useIgnoreLint(): (lint: Lint) => Promise<void> {
	const linter = useLinter();
	const [ignoredLintState, setIgnoredLintState] = useIgnoredLintState();

	return async (lint) => {
		await linter.clearIgnoredLints();

		if (ignoredLintState) {
			await linter.importIgnoredLints(ignoredLintState);
		}

		await linter.ignoreLint(lint);
		setIgnoredLintState(await linter.exportIgnoredLints());
	};
}
