import type { Lint } from 'harper.js';
import { useCallback, useEffect, useState } from 'react';
import type { IgnorableLintBox } from './Box';
import { useLinter } from './LinterProvider';
import type RichText from './RichText';
import useDialect from './useDialect';
import useIgnoredLintState, { useIgnoreLint } from './useIgnoredLintState';
import useLintConfig from './useLintConfig';
import usePersonalDictionary from './usePersonalDictionary';

/** Keep lints paired with their target and original source, even if the editor changes. */
type LintResult = {
	target: Element;
	source: string;
	lints: Lint[];
};

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
	const [lintResults, setLintResults] = useState<LintResult[]>([]);
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

		const newLints = await Promise.all(
			richTexts.map(async (richText) => {
				const source = richText.getTextContent();

				return { target: richText.getTargetElement(), source, lints: await linter.lint(source) };
			}),
		);

		setLoading(false);
		setLintResults(newLints);
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
			if (!running) return;

			const lintBoxes = richTexts.map((richText, index) => {
				const result = lintResults[index];
				if (
					result?.target !== richText.getTargetElement() ||
					result.source !== richText.getTextContent()
				)
					return [];

				return result.lints
					.flatMap((lint) => richText.computeLintBox(lint))
					.map((box) => {
						return {
							...box,
							ignoreLint: () => ignoreLint(result.source, box.lint),
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
	}, [lintResults, richTexts, ignoreLint]);

	return [targetBoxes, loading];
}
