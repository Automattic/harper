import { SearchControl } from '@wordpress/components';
import React, { useState } from 'react';
import { useLintDescriptions } from './LinterProvider';
import LintSettingRow from './LintSettingRow';
import useLintConfig, { useDefaultLintConfig } from './useLintConfig';

export default function LintSettingList() {
	const [lintConfig, setLintConfig] = useLintConfig();
	const defaultConfig = useDefaultLintConfig();
	const descriptions = useLintDescriptions();
	const [query, setQuery] = useState('');

	return (
		<div className="harper-lint-config-cont">
			<SearchControl value={query} onChange={setQuery} placeholder="Search for a rule..." />

			{Object.entries(lintConfig)
				.filter(([key]) => key.includes(query) || descriptions[key]?.includes(query))
				.map(([key, value]) => (
					<LintSettingRow
						key={key}
						name={key}
						description={descriptions[key]}
						value={value}
						defaultValue={defaultConfig[key]!}
						setValue={(newValue) => setLintConfig({ ...lintConfig, [key]: newValue })}
					/>
				))}
		</div>
	);
}
