import '@webcomponents/custom-elements';
import $ from 'jquery';
import LintFramework from '../LintFramework';
import { isVisible, leafNodes } from '../domUtils';

const fw = new LintFramework();

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
new MutationObserver(scan).observe(document.documentElement, { childList: true, subtree: true });
