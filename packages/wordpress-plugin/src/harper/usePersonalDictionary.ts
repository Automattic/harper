import { useDispatch, useSelect } from '@wordpress/data';
import { useCallback } from 'react';
import type { PreferencesActions, PreferencesSelectors } from './preferencesStoreTypes';

const KEY = 'personalDictionary';

/** Read and add to the user's personal dictionary. */
export default function usePersonalDictionary(): [
	string[] | undefined,
	(updatedDictionary: string[]) => void,
] {
	const personalDictionary = useSelect(
		(select) => (select('core/preferences') as PreferencesSelectors).get('harper-wp', KEY),
		[],
	);

	const { set } = useDispatch('core/preferences') as PreferencesActions;

	const updateState = useCallback(
		(updatedDictionary: string[]) => set('harper-wp', KEY, updatedDictionary),
		[set],
	);

	return [personalDictionary, updateState];
}

/** Get a callback that adds a word to the personal dictionary. */
export function useAddToDictionary(): (word: string) => void {
	const [dict, setDict] = usePersonalDictionary();

	return useCallback(
		(word: string) => {
			if (!dict?.includes(word)) {
				setDict([...(dict ?? []), word]);
			}
		},
		[dict, setDict],
	);
}
