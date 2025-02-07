import { useCallback, useEffect, useState } from 'react';
import { IgnorableLintBox, LintBox } from './Box';
import RichText from './RichText';
import { Lint } from 'harper.js';
import { useLinter } from './LinterProvider';
import useLintConfig from './useLintConfig';
import useIgnoredLintState, { useIgnoreLint } from './useIgnoredLintState';

/**
 * Lint given elements and return the resulting error targets.
 * Provides a loading state as well.
 * @param richTexts
 */
export default function useLintBoxes(
	richTexts: RichText[]
): [IgnorableLintBox[][], boolean] {
	const linter = useLinter();
	const [config] = useLintConfig();
	const [ignoreState] = useIgnoredLintState();
	const ignoreLint = useIgnoreLint();

	const [targetBoxes, setTargetBoxes] = useState<IgnorableLintBox[][]>([]);
	const [lints, setLints] = useState<Lint[][]>([]);
	const [loading, setLoading] = useState(true);

	const updateLints = useCallback(async () => {
		await linter.clearIgnoredLints();

		const tasks = [linter.setLintConfig(config)];

		if (ignoreState) {
			tasks.push(linter.importIgnoredLints(ignoreState));
		}

		await Promise.all(tasks);

		// We assume that a given index always refers to the same rich text field.
		const newLints = await Promise.all(
			richTexts.map(async (richText) => {
				const contents = richText.getTextContent();

				return await linter.lint(contents);
			})
		);

		setLoading(false);
		setLints(newLints);
	}, [richTexts, linter, config, ignoreState]);

	useEffect(() => {
		updateLints();

		const observers = richTexts.map((richText) => {
			const observer = new MutationObserver(updateLints);
			observer.observe(richText.getTargetElement(), {
				childList: true,
				characterData: true,
				subtree: true,
			});
			return observer;
		});

		return () => {
			observers.forEach((observer) => observer.disconnect());
		};
	}, [richTexts, updateLints]);

	// Update the lint boxes each frame.
	// Probably overkill.
	//
	// TODO: revisit this to do more lazily.
	// Maybe `onLayoutEffect`?
	useEffect(() => {
		let running = true;

		function onFrame() {
			const lintBoxes = lints.map((lintForText, index) => {
				const richText = richTexts[index];
				return lintForText
					.flatMap((lint) => richText.computeLintBox(lint))
					.map((box) => {
						return {
							...box,
							ignoreLint: () => ignoreLint(box.lint),
						};
					});
			});

			setTargetBoxes(lintBoxes);

			if (running) {
				requestAnimationFrame(onFrame);
			}
		}

		requestAnimationFrame(onFrame);

		return () => {
			running = false;
		};
	}, [lints, richTexts, ignoreLint]);

	return [targetBoxes, loading];
}
