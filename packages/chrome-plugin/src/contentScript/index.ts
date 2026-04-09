import '@webcomponents/custom-elements';
import {
	getClosestBlockAncestor,
	isVisible,
	LintFramework,
	leafNodes,
	type UnpackedLint,
} from 'lint-framework';
import isSubstack from '../isSubstack';
import isWordPress from '../isWordPress';
import ProtocolClient from '../ProtocolClient';
import { createGoogleDocsBridgeSync, isGoogleDocsPage } from './googleDocs';

if (isWordPress() || isSubstack()) {
	ProtocolClient.setDomainEnabled(window.location.hostname, true, false);
}

const fw = new LintFramework(
	(text, domain, options) => ProtocolClient.lint(text, domain, options),
	{
		ignoreLint: (hash) => ProtocolClient.ignoreHash(hash),
		getActivationKey: () => ProtocolClient.getActivationKey(),
		getHotkey: () => ProtocolClient.getHotkey(),
		openOptions: () => ProtocolClient.openOptions(),
		addToUserDictionary: (words) => ProtocolClient.addToUserDictionary(words),
		reportError: (lint: UnpackedLint, ruleId: string) =>
			ProtocolClient.openReportError(
				padWithContext(lint.source, lint.span.start, lint.span.end, 15),
				ruleId,
				'',
			),
		setRuleEnabled: async (ruleId, enabled) => {
			await ProtocolClient.setRuleEnabled(ruleId, enabled);
			fw.update();
		},
	},
);

const syncGoogleDocsBridge = createGoogleDocsBridgeSync(fw);

function padWithContext(source: string, start: number, end: number, contextLength: number): string {
	const normalizedStart = Math.max(0, Math.min(start, source.length));
	const normalizedEnd = Math.max(normalizedStart, Math.min(end, source.length));
	const contextStart = Math.max(0, normalizedStart - contextLength);
	const contextEnd = Math.min(source.length, normalizedEnd + contextLength);

	return source.slice(contextStart, contextEnd);
}

const keepAliveCallback = () => {
	ProtocolClient.lint('', 'example.com', {});
	void syncGoogleDocsBridge();

	setTimeout(keepAliveCallback, 400);
};

keepAliveCallback();

/**
 * Returns the reasons Harper should skip this textarea while scanning for targets.
 * An empty array means the textarea is eligible to be added.
 */
function getTextareaReasons(element: HTMLTextAreaElement, requireVisible: boolean): string[] {
	const reasons = [];

	if (requireVisible && !isVisible(element)) {
		reasons.push('not-visible');
	}

	if (element.getAttribute('data-enable-grammarly') === 'false') {
		reasons.push('grammarly-disabled');
	}

	if (element.disabled) {
		reasons.push('disabled');
	}

	if (element.readOnly) {
		reasons.push('readonly');
	}

	return reasons;
}

function maybeAddTextareaTarget(element: HTMLTextAreaElement, requireVisible: boolean) {
	const reasons = getTextareaReasons(element, requireVisible);
	if (reasons.length > 0) {
		return;
	}

	fw.addTarget(element);
}

function scan() {
	void syncGoogleDocsBridge();

	if (isGoogleDocsPage()) {
		return;
	}

	document.querySelectorAll<HTMLTextAreaElement>('textarea').forEach((element) => {
		maybeAddTextareaTarget(element, true);
	});

	document
		.querySelectorAll<HTMLInputElement>('input[type="text"][spellcheck="true"]')
		.forEach((element) => {
			if (element.disabled || element.readOnly) {
				return;
			}

			fw.addTarget(element);
		});

	document.querySelectorAll('[data-testid="gutenberg-editor"]').forEach((element) => {
		const leafs = leafNodes(element);

		const seenBlockContainers = new Set<Element>();

		for (const leaf of leafs) {
			const blockContainer = getClosestBlockAncestor(leaf, element);

			if (!blockContainer || seenBlockContainers.has(blockContainer)) {
				continue;
			}

			seenBlockContainers.add(blockContainer);

			if (!isVisible(blockContainer)) {
				continue;
			}

			fw.addTarget(blockContainer);
		}
	});

	document
		.querySelectorAll<HTMLElement>('.cm-editor .cm-content[contenteditable="true"]')
		.forEach((element) => {
			const isTypstPlayground = window.location.hostname === 'typst.app';
			const explicitlyTypst = element.getAttribute('data-language') === 'typst';

			if (!isTypstPlayground && !explicitlyTypst) {
				return;
			}

			if (element.closest('[contenteditable="false"],[disabled],[readonly]') != null) {
				return;
			}

			if (!isVisible(element)) {
				return;
			}

			element.setAttribute('data-language', 'typst');
			fw.addTarget(element);
		});

	document.querySelectorAll('[contenteditable="true"],[contenteditable]').forEach((element) => {
		if (element.classList.contains('cm-content') && element.closest('.cm-editor') != null) {
			return;
		}

		const isLexicalEditor = element.getAttribute('data-lexical-editor') === 'true';

		if (
			element.matches('[role="combobox"]') ||
			element.getAttribute('data-enable-grammarly') === 'false' ||
			(element.getAttribute('spellcheck') === 'false' &&
				!isLexicalEditor &&
				element.getAttribute('data-language') !== 'markdown')
		) {
			return;
		}

		if (element.classList.contains('ck-editor__editable')) {
			element.querySelectorAll('p').forEach((paragraph) => {
				if (paragraph.closest('[contenteditable="false"],[disabled],[readonly]') != null) {
					return;
				}

				if (!isVisible(paragraph)) {
					return;
				}

				fw.addTarget(paragraph);
			});

			return;
		}

		const leafs = leafNodes(element);

		const seenBlockContainers = new Set<Element>();

		for (const leaf of leafs) {
			if (leaf.parentElement?.closest('[contenteditable="false"],[disabled],[readonly]') != null) {
				continue;
			}

			const blockContainer = getClosestBlockAncestor(leaf, element);

			if (!blockContainer || seenBlockContainers.has(blockContainer)) {
				continue;
			}

			seenBlockContainers.add(blockContainer);

			if (!isVisible(blockContainer)) {
				continue;
			}

			fw.addTarget(blockContainer);
		}
	});
}

scan();
new MutationObserver(scan).observe(document.body, {
	childList: true,
	subtree: true,
});

document.addEventListener(
	'focusin',
	(event) => {
		const target = event.target;
		if (!(target instanceof HTMLTextAreaElement)) {
			return;
		}

		maybeAddTextareaTarget(target, false);
	},
	{ capture: true },
);

setTimeout(scan, 1000);
