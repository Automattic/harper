import type { Lint } from 'harper.js';
import { useCallback, useEffect, useState } from 'react';
import type { IgnorableLintBox } from './Box';
import { useLinter } from './LinterProvider';
import type RichText from './RichText';
import useDialect from './useDialect';
import useIgnoredLintState, { useIgnoreLint } from './useIgnoredLintState';
import useLintConfig from './useLintConfig';
import usePersonalDictionary from './usePersonalDictionary';

/**
 * Lint given elements and return the resulting error targets.
 * Provides a loading state as well.
 * @param richTexts
 */
export default function useLintBoxes(richTexts: RichText[]): [IgnorableLintBox[][], boolean] {
	const linter = useLinter();
	const [config] = useLintConfig();
	const [dialect] = useDialect();
	const [ignoreState] = useIgnoredLintState();
	const [personalDictionary] = usePersonalDictionary();
	const ignoreLint = useIgnoreLint();

	const [targetBoxes, setTargetBoxes] = useState<IgnorableLintBox[][]>([]);
	const [lints, setLints] = useState<Lint[][]>([]);
	const [loading, setLoading] = useState(true);

	const updateLints = useCallback(async () => {
		if ((await linter.exportIgnoredLints()) !== ignoreState) {
			await linter.clearIgnoredLints();
		}

		console.log(dialect);

		await linter.setDialect(dialect);

		if (personalDictionary) {
			await linter.importWords(personalDictionary);
		}

		if (JSON.stringify(await linter.getLintConfig()) !== JSON.stringify(config)) {
			await linter.setLintConfig(config);
		}

		if (ignoreState) {
			await linter.importIgnoredLints(ignoreState);
		}

		// We assume that a given index always refers to the same rich text field.
		const newLints = await Promise.all(
			richTexts.map(async (richText) => {
				const contents = richText.getTextContent();

				return await linter.lint(contents);
			}),
		);

		setLoading(false);
		setLints(newLints);
	}, [richTexts, linter, config, ignoreState, personalDictionary, dialect]);

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
			observers.forEach((observer) => {
				observer.disconnect();
			});
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
