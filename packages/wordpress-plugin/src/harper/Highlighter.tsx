import { useEffect } from 'react';
import type { LintBox } from './Box';
import type RichText from './RichText';
import SuggestionControl from './SuggestionControl';

/**
 * Renders controls to the user around the errors.
 * @param root0
 * @param root0.lintBoxes
 * @param root0.richText
 */
export default function Highlighter({
	lintBoxes,
	richText,
}: {
	lintBoxes: LintBox[];
	richText: RichText;
}) {
	// Disable browser spellchecking in favor of ours
	useEffect(() => {
		richText.getTargetElement().spellcheck = false;

		return () => {
			richText.getTargetElement().spellcheck = true;
		};
	}, [richText]);

	const visible = richText.getTargetElement().checkVisibility();

	return (
		<>{visible && lintBoxes.map((b, index) => <SuggestionControl lintBox={b} key={index} />)}</>
	);
}
