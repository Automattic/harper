import { useDispatch, useSelect } from '@wordpress/data';
import { Dialect } from 'harper.js';
import { useCallback, useMemo } from 'react';
import type { PreferencesActions, PreferencesSelectors } from './preferencesStoreTypes';

const KEY = 'dialect';

export default function useDialect(): [Dialect, (newState: Dialect) => void] {
	const dialect = useSelect(
		(select) => (select('core/preferences') as PreferencesSelectors).get('harper-wp', KEY),
		[],
	);

	const { set } = useDispatch('core/preferences') as PreferencesActions;

	const setConfig = useCallback((newValue: Dialect) => {
		set('harper-wp', KEY, newValue);
	}, []);

	const nonNull = useMemo(() => {
		if (dialect == null) {
			return Dialect.American;
		}
		return dialect;
	}, [dialect]);

	return [nonNull, setConfig];
}
