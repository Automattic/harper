import { useDispatch, useSelect } from '@wordpress/data';
import type { LintConfig } from 'harper.js';
import { merge } from 'lodash-es';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useLinter } from './LinterProvider';

const KEY = 'lintConfig';

export default function useLintConfig(): [LintConfig, (newState: LintConfig) => void] {
	const defaultConfig = useDefaultLintConfig();
	const lintConfig = useSelect((select) => select('core/preferences').get('harper-wp', KEY), []);

	const { set } = useDispatch('core/preferences');

	const setConfig = useCallback((newValue) => {
		set('harper-wp', KEY, newValue);
	}, []);

	useEffect(() => {
		if (
			lintConfig == null ||
			Object.entries(lintConfig).length < Object.entries(defaultConfig).length
		) {
			merge(lintConfig, defaultConfig);
			setConfig({ ...lintConfig });
		}
	}, [defaultConfig, setConfig]);

	const nonNull = useMemo(() => {
		if (lintConfig == null) {
			return defaultConfig;
		}
		return lintConfig;
	}, [lintConfig]);

	return [nonNull, setConfig];
}

export function useDefaultLintConfig(): LintConfig {
	const linter = useLinter();
	const [defaultConfig, setDefaultConfig] = useState({});

	useEffect(() => {
		linter.getDefaultLintConfig().then(setDefaultConfig);
	}, [linter]);

	return defaultConfig;
}
