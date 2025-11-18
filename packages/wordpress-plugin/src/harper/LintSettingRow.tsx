import { Animate, CheckboxControl } from '@wordpress/components';
import { useEffect, useState } from 'react';
import { useLinter } from './LinterProvider';

export default function LintSettingRow({
	name,
	value,
	defaultValue,
	setValue,
	description,
}: {
	name: string;
	description: string;
	value: boolean | undefined;
	defaultValue: boolean;
	setValue: (newValue: boolean | undefined) => void;
}) {
	const linter = useLinter();

	const [title, setTitle] = useState<string | null>(null);

	useEffect(() => {
		linter.toTitleCase(name.replace(/_/g, ' ')).then(setTitle);
	}, [linter, name]);

	return title && description ? (
		<Animate type={title === null ? undefined : 'slide-in'}>
			{({ className }) => (
				<div className={`${className} harper-lint-config-row`}>
					<h3>{title}</h3>
					<p>{description}</p>

					<CheckboxControl
						label={
							value == null
								? `Default (${defaultValue ? 'Enabled' : 'Disabled'})`
								: value
									? 'Enabled'
									: 'Disabled'
						}
						onChange={(val) => setValue(val)}
						checked={value ?? defaultValue}
					/>
				</div>
			)}
		</Animate>
	) : (
		<></>
	);
}
