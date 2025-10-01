import '@webcomponents/custom-elements';
import $ from 'jquery';
import { isVisible, LintFramework, leafNodes } from 'lint-framework';
import ProtocolClient from '../ProtocolClient';

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
	$('textarea:visible').each(function () {
		if (this.getAttribute('data-enable-grammarly') == 'false' || this.disabled || this.readOnly) {
			return;
		}

		fw.addTarget(this as HTMLTextAreaElement);
	});

	$('input[type="text"][spellcheck="true"]').each(function () {
		if (this.disabled || this.readOnly) {
			return;
		}

		fw.addTarget(this as HTMLInputElement);
	});

	$('[data-testid="gutenberg-editor"]').each(function () {
		const leafs = leafNodes(this);

		for (const leaf of leafs) {
			if (!isVisible(leaf)) {
				continue;
			}

			fw.addTarget(leaf);
		}
	});

	$('[contenteditable="true"],[contenteditable]').each(function () {
		const leafs = leafNodes(this);

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
