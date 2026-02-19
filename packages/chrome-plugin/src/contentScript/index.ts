import '@webcomponents/custom-elements';
import {
	getClosestBlockAncestor,
	isVisible,
	LintFramework,
	leafNodes,
	type UnpackedLint,
} from 'lint-framework';
import isWordPress from '../isWordPress';
import ProtocolClient from '../ProtocolClient';

if (isWordPress()) {
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

const GOOGLE_DOCS_BRIDGE_ID = 'harper-google-docs-target';
const GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID = 'harper-google-docs-main-world-bridge';
const GOOGLE_DOCS_LAYOUT_EPOCH_ATTR = 'data-harper-layout-epoch';
const GOOGLE_DOCS_TEXT_UPDATED_EVENT = 'harper:gdocs:text-updated';

let googleDocsSyncInFlight = false;
let googleDocsSyncPending = false;
let googleDocsBridgeAttached = false;
let googleDocsEventsBound = false;
let googleDocsSyncScheduled = false;
let googleDocsFrameRefreshStarted = false;
let googleDocsLastLayoutEpoch = '';

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

function scan() {
	void syncGoogleDocsBridge();

	if (isGoogleDocsPage()) {
		return;
	}

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

	document.querySelectorAll('[contenteditable="true"],[contenteditable]').forEach((element) => {
		if (
			element.matches('[role="combobox"]') ||
			element.getAttribute('data-enable-grammarly') === 'false' ||
			(element.getAttribute('spellcheck') === 'false' &&
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

function isGoogleDocsPage(): boolean {
	return window.location.hostname === 'docs.google.com' && window.location.pathname.startsWith('/document/');
}

function getGoogleDocsBridge(): HTMLElement {
	let bridge = document.getElementById(GOOGLE_DOCS_BRIDGE_ID);

	if (!bridge) {
		bridge = document.createElement('div');
		bridge.id = GOOGLE_DOCS_BRIDGE_ID;
		bridge.setAttribute('data-harper-google-docs-target', 'true');
		bridge.setAttribute('aria-hidden', 'true');
		bridge.style.position = 'fixed';
		bridge.style.top = '0';
		bridge.style.left = '0';
		bridge.style.width = '1px';
		bridge.style.height = '1px';
		bridge.style.overflow = 'hidden';
		bridge.style.zIndex = '-2147483648';
		(document.body || document.documentElement).appendChild(bridge);
	}

	return bridge;
}

function ensureGoogleDocsMainWorldBridge() {
	if (document.getElementById(GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID)) {
		return;
	}

	const script = document.createElement('script');
	script.src = chrome.runtime.getURL('google-docs-bridge.js');
	(document.head || document.documentElement).appendChild(script);
	script.onload = () => script.remove();
}

function scheduleGoogleDocsBridgeSync() {
	if (googleDocsSyncScheduled) {
		return;
	}

	googleDocsSyncScheduled = true;

	setTimeout(async () => {
		googleDocsSyncScheduled = false;
		await syncGoogleDocsBridge();
		await fw.update();
	}, 0);
}

function startGoogleDocsFrameRefreshLoop() {
	if (googleDocsFrameRefreshStarted) {
		return;
	}

	googleDocsFrameRefreshStarted = true;

	const tick = () => {
		if (!isGoogleDocsPage()) {
			googleDocsFrameRefreshStarted = false;
			googleDocsLastLayoutEpoch = '';
			return;
		}

		const bridge = document.getElementById(GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID);
		const layoutEpoch = bridge?.getAttribute(GOOGLE_DOCS_LAYOUT_EPOCH_ATTR) ?? '';

		if (layoutEpoch !== googleDocsLastLayoutEpoch) {
			googleDocsLastLayoutEpoch = layoutEpoch;
			fw.refreshLayout();
		}

		requestAnimationFrame(tick);
	};

	requestAnimationFrame(tick);
}

function bindGoogleDocsBridgeEvents() {
	if (googleDocsEventsBound || !isGoogleDocsPage()) {
		return;
	}

	googleDocsEventsBound = true;
	document.addEventListener(GOOGLE_DOCS_TEXT_UPDATED_EVENT, scheduleGoogleDocsBridgeSync);
	startGoogleDocsFrameRefreshLoop();
}

async function syncGoogleDocsBridge() {
	if (!isGoogleDocsPage()) {
		return;
	}

	if (googleDocsSyncInFlight) {
		googleDocsSyncPending = true;
		return;
	}

	googleDocsSyncInFlight = true;

	try {
		ensureGoogleDocsMainWorldBridge();
		bindGoogleDocsBridgeEvents();

		const bridge = getGoogleDocsBridge();
		const mainWorldBridge = document.getElementById(GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID);
		const text = mainWorldBridge?.textContent ?? '';

		if (bridge.textContent !== text) {
			bridge.textContent = text;
		}

		if (!googleDocsBridgeAttached) {
			await fw.addTarget(bridge);
			googleDocsBridgeAttached = true;
		}
	} catch (err) {
		console.error('Failed to sync Google Docs bridge text', err);
	} finally {
		googleDocsSyncInFlight = false;

		if (googleDocsSyncPending) {
			googleDocsSyncPending = false;
			void syncGoogleDocsBridge();
		}
	}
}

scan();
new MutationObserver(scan).observe(document.body, { childList: true, subtree: true });

setTimeout(scan, 1000);
