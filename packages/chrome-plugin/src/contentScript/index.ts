import '@webcomponents/custom-elements';
import $ from 'jquery';
import LintFramework from '../LintFramework';
import { leafNodes } from '../domUtils';

const fw = new LintFramework();

function scan() {
	$('textarea:visible').each(function () {
		if (this.getAttribute('data-enable-grammarly') == 'false') {
			return;
		}

		fw.addTarget(this as HTMLTextAreaElement);
	});

	$('input[type="text"][spellcheck="true"]').each(function () {
		fw.addTarget(this as HTMLInputElement);
	});

	$('[contenteditable="true"],[contenteditable]').each(function () {
		const leafs = leafNodes(this);

		for (const leaf of leafs) {
			fw.addTarget(leaf as HTMLElement);
		}
	});
}

scan();
new MutationObserver(scan).observe(document.documentElement, { childList: true, subtree: true });
