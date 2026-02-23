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
const GOOGLE_DOCS_SVG_RECT_SELECTOR = 'rect[aria-label]';

let googleDocsSyncInFlight = false;
let googleDocsSyncPending = false;
let googleDocsBridgeAttached = false;
let googleDocsFrameRefreshStarted = false;
let googleDocsLastLayoutEpoch = '';
let googleDocsCloneSignature = '';

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
	if (!isGoogleDocsPage()) {
		return;
	}

	startGoogleDocsFrameRefreshLoop();
}

function normalizeGoogleDocsLabel(label: string): string {
	const tokens = label.split(' ');

	for (let i = 0; i < tokens.length; i += 1) {
		const token = tokens[i];
		if (token === '') {
			tokens[i] = '\xa0';
			continue;
		}

		if (token.length === 1 && !token.match(/[a-zA-Z]/)) {
			tokens[i] = ` ${token} `;
			continue;
		}

		const isLast = i === tokens.length - 1;
		const lastChar = token.charAt(token.length - 1);
		const nextFirstChar = tokens[i + 1]?.charAt(0) ?? '';
		const keepTightTrailing = /[\(\["'“\-_`]/.test(lastChar);
		const keepTightLeadingNext = /[\)\]"'”\-_`]/.test(nextFirstChar);

		tokens[i] = !isLast && !keepTightTrailing && !keepTightLeadingNext ? `${token} ` : token;
	}

	return tokens.join('');
}

function addHashToken(hash: number, token: string): number {
	let next = hash;
	for (let i = 0; i < token.length; i += 1) {
		next = (next * 33 + token.charCodeAt(i)) >>> 0;
	}
	return next;
}

function rebuildGoogleDocsClone(editor: HTMLElement, clone: HTMLElement): { changed: boolean } {
	const rectNodes = editor.querySelectorAll<SVGRectElement>(GOOGLE_DOCS_SVG_RECT_SELECTOR);
	const editorRect = editor.getBoundingClientRect();
	const scrollTop = editor.scrollTop;
	const scrollLeft = editor.scrollLeft;
	const fragment = document.createDocumentFragment();
	let nextHash = 5381;
	let segmentCount = 0;
	let textLength = 0;

	for (const rectNode of Array.from(rectNodes)) {
		const areaLabel = rectNode.getAttribute('aria-label');
		if (!areaLabel) continue;

		const normalizedLabel = normalizeGoogleDocsLabel(areaLabel);
		if (!normalizedLabel) continue;

		const rect = rectNode.getBoundingClientRect();
		if (!Number.isFinite(rect.top) || rect.width <= 0 || rect.height <= 0) continue;

		const top = rect.top - editorRect.top + scrollTop;
		const left = rect.left - editorRect.left + scrollLeft;
		if (!Number.isFinite(top) || !Number.isFinite(left)) continue;

		const span = document.createElement('span');
		span.textContent = normalizedLabel;
		span.style.position = 'absolute';
		span.style.whiteSpace = 'pre';
		span.style.overflow = 'hidden';
		span.style.left = `${left}px`;
		span.style.top = `${top}px`;
		span.style.width = `${Math.max(rect.width, 1)}px`;
		span.style.height = `${Math.max(rect.height, 1)}px`;
		span.style.lineHeight = `${Math.max(rect.height, 1)}px`;
		const fontCss = rectNode.getAttribute('data-font-css');
		if (fontCss) {
			span.style.font = fontCss;
		}

		fragment.appendChild(span);

		nextHash = addHashToken(nextHash, normalizedLabel);
		nextHash = addHashToken(
			nextHash,
			`${Math.round(left)}:${Math.round(top)}:${Math.round(rect.width)}:${Math.round(rect.height)}`,
		);
		segmentCount += 1;
		textLength += normalizedLabel.length;
	}

	nextHash = addHashToken(nextHash, `${segmentCount}:${textLength}:${Math.round(scrollTop)}`);
	const nextSignature = String(nextHash);
	if (nextSignature === googleDocsCloneSignature && clone.childNodes.length > 0) {
		return { changed: false };
	}

	clone.replaceChildren(fragment);
	clone.setAttribute('data-harper-gdocs-segments', String(segmentCount));
	googleDocsCloneSignature = nextSignature;
	return { changed: true };
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
		const bridge = getGoogleDocsBridge(editor);
		const { changed } = rebuildGoogleDocsClone(editor, bridge);

		if (!googleDocsBridgeAttached) {
			await fw.addTarget(bridge);
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
