import '@webcomponents/custom-elements';
import { isVisible, LintFramework, leafNodes } from 'lint-framework';
import isWordPress from '../isWordPress';
import ProtocolClient from '../ProtocolClient';

if (isWordPress()) {
	ProtocolClient.setDomainEnabled(window.location.hostname, true);
}

const fw = new LintFramework((text, domain) => ProtocolClient.lint(text, domain), {
	ignoreLint: (hash) => ProtocolClient.ignoreHash(hash),
	getActivationKey: () => ProtocolClient.getActivationKey(),
	openOptions: () => ProtocolClient.openOptions(),
	addToUserDictionary: (words) => ProtocolClient.addToUserDictionary(words),
});

const keepAliveCallback = () => {
	ProtocolClient.lint('', 'example.com');

	setTimeout(keepAliveCallback, 400);
};

keepAliveCallback();

function scan() {
	document.querySelectorAll<HTMLTextAreaElement>('textarea').forEach((element) => {
		if (
			!isVisible(element) ||
			element.getAttribute('data-enable-grammarly') === 'false' ||
			element.disabled ||
			element.readOnly
		) {
			return;
		}

		fw.addTarget(element);
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

		for (const leaf of leafs) {
			if (!isVisible(leaf)) {
				continue;
			}

			fw.addTarget(leaf);
		}
	});

	document.querySelectorAll('[contenteditable="true"],[contenteditable]').forEach((element) => {
		if (element.matches('[role="combobox"]')) {
			return;
		}

		const leafs = leafNodes(element);

		for (const leaf of leafs) {
			if (leaf.parentElement?.closest('[contenteditable="false"],[disabled],[readonly]') != null) {
				continue;
			}

			if (!isVisible(leaf)) {
				continue;
			}

			fw.addTarget(leaf);
		}
	});
}

scan();
new MutationObserver(scan).observe(document.body, { childList: true, subtree: true });

setTimeout(scan, 1000);
