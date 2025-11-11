import { Button, Card, CardBody } from '@wordpress/components';
import type { IgnorableLintBox } from './Box';
import { suggestionText } from './lintUtils';
import { useAddToDictionary } from './usePersonalDictionary';

export default function LintListItem({ box }: { box: IgnorableLintBox }) {
	const addToDictionary = useAddToDictionary();

	return (
		<Card size="small" className="harper-lint-card">
			<CardBody>
				<h2 className={`harper-underline-${box.lint.lint_kind()}`}>
					{box.lint.lint_kind_pretty()}
				</h2>
				<p>{box.lint.message()}</p>

				{box.lint.suggestions().map((sug, index) => (
					<Button variant="primary" key={index} onClick={() => box.applySuggestion(sug)}>
						{suggestionText(sug.kind(), box.lint.get_problem_text(), sug.get_replacement_text())}
					</Button>
				))}

				{box.lint.lint_kind() === 'Spelling' ? (
					<Button onClick={() => addToDictionary(box.lint.get_problem_text())} variant="primary">
						Add “{box.lint.get_problem_text()}” to the dictionary
					</Button>
				) : (
					<></>
				)}

				<Button variant="link" onClick={box.ignoreLint}>
					Ignore
				</Button>
			</CardBody>
		</Card>
	);
}
