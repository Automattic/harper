import { useDispatch, useSelect } from '@wordpress/data';
import { useCallback, useMemo } from 'react';

const KEY = 'langiso639';

export default function useIsoLang639(): [string, (newState: string) => void] {
	const lang = useSelect((select) => select('core/preferences').get('harper-wp', KEY), []);
	const { set } = useDispatch('core/preferences');

	const setConfig = useCallback((newValue) => {
		set('harper-wp', KEY, newValue);
	}, []);

	const nonNull = useMemo(() => {
		if (lang == null) {
			return 'en'; // Default to English
		}
		return lang;
	}, [lang]);

	return [nonNull, setConfig];
}