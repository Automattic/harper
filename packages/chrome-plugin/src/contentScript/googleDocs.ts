import type { LintFramework } from 'lint-framework';
import GoogleDocsBridgeClient from './GoogleDocsBridgeClient';

declare global {
	interface Window {
		__harperGoogleDocsBridgeClient?: GoogleDocsBridgeClient;
	}
}

const GOOGLE_DOCS_BRIDGE_ID = 'harper-google-docs-target';
const GOOGLE_DOCS_MAIN_WORLD_BRIDGE_ID = 'harper-google-docs-main-world-bridge';
const GOOGLE_DOCS_SCROLL_LAYOUT_REASONS = new Set(['scroll', 'wheel', 'key-scroll']);
const GOOGLE_DOCS_EDITOR_SELECTOR = '.kix-appview-editor';
const GOOGLE_DOCS_SVG_RECT_SELECTOR = 'rect[aria-label]';
const GOOGLE_DOCS_LINE_BREAK_THRESHOLD_PX = 6;

type LayoutRefreshFramework = LintFramework & {
	refreshLayout?: () => void;
};

export function isGoogleDocsPage(): boolean {
	return (
		window.location.hostname === 'docs.google.com' &&
		window.location.pathname.startsWith('/document/')
	);
}

export function createGoogleDocsBridgeSync(fw: LintFramework): () => Promise<void> {
	let googleDocsSyncInFlight = false;
	let googleDocsSyncPending = false;
	let googleDocsBridgeAttached = false;
	let googleDocsEventsBound = false;
	let googleDocsCloneSignature = '';
	let googleDocsBridgeClient: GoogleDocsBridgeClient | null = null;

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
		script.type = 'module';
		script.src = chrome.runtime.getURL('google-docs-bridge.js');
		(document.head || document.documentElement).appendChild(script);
		script.onload = () => script.remove();
	}

	function bindGoogleDocsBridgeEvents(syncGoogleDocsBridge: () => Promise<void>) {
		if (!isGoogleDocsPage() || googleDocsEventsBound) {
			return;
		}

		googleDocsEventsBound = true;
		googleDocsBridgeClient = new GoogleDocsBridgeClient(document);
		window.__harperGoogleDocsBridgeClient = googleDocsBridgeClient;
		googleDocsBridgeClient.onTextUpdated(() => {
			void syncGoogleDocsBridge();
		});
		googleDocsBridgeClient.onLayoutChanged((reason) => {
			if (!GOOGLE_DOCS_SCROLL_LAYOUT_REASONS.has(String(reason))) {
				(fw as LayoutRefreshFramework).refreshLayout?.();
			}
		});
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
			const keepTightTrailing = /[(["'“\-_`]/.test(lastChar);
			const keepTightLeadingNext = /[)\]"'”\-_`]/.test(nextFirstChar);

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
		const parts: string[] = [];
		let nextHash = 5381;
		let lastTop: number | null = null;
		let segmentCount = 0;

		for (const rectNode of Array.from(rectNodes)) {
			const areaLabel = rectNode.getAttribute('aria-label');
			if (!areaLabel) continue;

			const normalizedLabel = normalizeGoogleDocsLabel(areaLabel);
			if (!normalizedLabel) continue;

			const rect = rectNode.getBoundingClientRect();
			if (!Number.isFinite(rect.top) || rect.width <= 0 || rect.height <= 0) continue;

			const top = rect.top - editorRect.top + scrollTop;
			const left = rect.left - editorRect.left + scrollLeft;
			if (!Number.isFinite(top)) continue;
			if (!Number.isFinite(left)) continue;

			if (lastTop != null && Math.abs(top - lastTop) >= GOOGLE_DOCS_LINE_BREAK_THRESHOLD_PX) {
				if (parts.length > 0 && !parts[parts.length - 1].endsWith('\n')) {
					parts.push('\n');
					fragment.appendChild(document.createTextNode('\n'));
				}
			}

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

			parts.push(normalizedLabel);
			lastTop = top;
			segmentCount += 1;

			nextHash = addHashToken(nextHash, normalizedLabel);
			nextHash = addHashToken(
				nextHash,
				`${Math.round(top)}:${Math.round(left)}:${Math.round(rect.width)}`,
			);
		}

		const nextText = parts.join('');
		nextHash = addHashToken(
			nextHash,
			`${nextText.length}:${segmentCount}:${Math.round(scrollTop)}`,
		);
		const nextSignature = String(nextHash);
		if (nextSignature === googleDocsCloneSignature && clone.textContent === nextText) {
			return { changed: false };
		}

		clone.replaceChildren(fragment);
		clone.setAttribute('data-harper-gdocs-segments', String(segmentCount));
		googleDocsCloneSignature = nextSignature;
		return { changed: true };
	}

	async function syncGoogleDocsBridge() {
		if (!isGoogleDocsPage()) {
			if (googleDocsBridgeClient) {
				googleDocsBridgeClient.dispose();
				googleDocsBridgeClient = null;
			}
			delete window.__harperGoogleDocsBridgeClient;
			googleDocsEventsBound = false;
			return;
		}

		if (googleDocsSyncInFlight) {
			googleDocsSyncPending = true;
			return;
		}

		googleDocsSyncInFlight = true;

		try {
			ensureGoogleDocsMainWorldBridge();
			bindGoogleDocsBridgeEvents(syncGoogleDocsBridge);

			const editor = document.querySelector(GOOGLE_DOCS_EDITOR_SELECTOR);
			if (!(editor instanceof HTMLElement)) {
				return;
			}
			const target = getGoogleDocsBridge(editor);
			const { changed } = rebuildGoogleDocsClone(editor, target);

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

	return syncGoogleDocsBridge;
}
