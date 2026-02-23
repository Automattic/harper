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
if (isGoogleDocsPage()) {
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
const GOOGLE_DOCS_LAYOUT_REASON_ATTR = 'data-harper-layout-reason';
const GOOGLE_DOCS_SCROLL_LAYOUT_REASONS = new Set(['scroll', 'wheel', 'key-scroll']);
const GOOGLE_DOCS_EDITOR_SELECTOR = '.kix-appview-editor';
const GOOGLE_DOCS_TEXT_UPDATED_EVENT = 'harper:gdocs:text-updated';

let googleDocsSyncInFlight = false;
let googleDocsSyncPending = false;
let googleDocsBridgeAttached = false;
let googleDocsFrameRefreshStarted = false;
let googleDocsEventsBound = false;
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
	return (
		window.location.hostname === 'docs.google.com' &&
		window.location.pathname.startsWith('/document/')
	);
}

function getGoogleDocsBridge(editor: HTMLElement): HTMLElement {
	let bridge = document.getElementById(GOOGLE_DOCS_BRIDGE_ID);

	if (!bridge) {
		bridge = document.createElement('div');
		bridge.id = GOOGLE_DOCS_BRIDGE_ID;
		bridge.setAttribute('data-harper-google-docs-target', 'true');
		bridge.setAttribute('aria-hidden', 'true');
		bridge.style.position = 'absolute';
		bridge.style.top = '0';
		bridge.style.left = '0';
		bridge.style.width = '0';
		bridge.style.height = '0';
		bridge.style.overflow = 'visible';
		bridge.style.pointerEvents = 'none';
		bridge.style.opacity = '0';
		bridge.style.zIndex = '-2147483648';
		bridge.setAttribute('contenteditable', 'false');
		editor.appendChild(bridge);
	}

	if (bridge.parentElement !== editor) {
		editor.appendChild(bridge);
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
		const layoutReason = bridge?.getAttribute(GOOGLE_DOCS_LAYOUT_REASON_ATTR) ?? '';

		if (layoutEpoch !== googleDocsLastLayoutEpoch) {
			googleDocsLastLayoutEpoch = layoutEpoch;
			if (!GOOGLE_DOCS_SCROLL_LAYOUT_REASONS.has(layoutReason)) {
				(fw as any).refreshLayout?.();
			}
		}

		requestAnimationFrame(tick);
	};

	requestAnimationFrame(tick);
}

function bindGoogleDocsBridgeEvents() {
	if (!isGoogleDocsPage() || googleDocsEventsBound) {
		return;
	}

	googleDocsEventsBound = true;
	startGoogleDocsFrameRefreshLoop();
	document.addEventListener(GOOGLE_DOCS_TEXT_UPDATED_EVENT, () => {
		void syncGoogleDocsBridge();
	});
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

		const editor = document.querySelector(GOOGLE_DOCS_EDITOR_SELECTOR);
		if (!(editor instanceof HTMLElement)) {
			return;
		}
		const target = getGoogleDocsBridge(editor);
		const mainBridge = document.getElementById(GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID);
		const canonicalText = mainBridge?.textContent ?? '';
		const changed = target.textContent !== canonicalText;
		if (changed) {
			target.textContent = canonicalText;
		}

		if (!googleDocsBridgeAttached) {
			await fw.addTarget(target);
			googleDocsBridgeAttached = true;
		}

		if (changed) {
			await fw.update();
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
