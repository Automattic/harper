/**
 * In Firefox, a page with a strict Content Security Policy (`style-src` without
 * `'unsafe-inline'`) blocks the `<style>` elements we render into our shadow roots: they stay in
 * the DOM, but never get a stylesheet (`sheet` is `null`). Constructed stylesheets are not subject
 * to that policy, so we adopt the same CSS as a fallback.
 *
 * Firefox runs content scripts behind Xray wrappers: a stylesheet constructed in the content
 * script cannot be adopted by the page's shadow roots. There we construct the sheet and the array
 * with the page's own constructors (`wrappedJSObject`).
 */

type Waivable = { wrappedJSObject?: unknown };

/** The page's view of an object in Firefox content scripts, the object itself elsewhere. */
function pageView<T>(object: T): any {
	return (object as Waivable).wrappedJSObject ?? object;
}

const sheetsByCss = new Map<string, CSSStyleSheet>();

/** Whether the browser refused to apply the given `<style>` element. */
export function isStyleBlocked(style: HTMLStyleElement): boolean {
	return style.isConnected && style.sheet == null;
}

/** A constructed stylesheet with the given CSS, shared between callers. */
export function constructedSheet(css: string): CSSStyleSheet | null {
	const Sheet = pageView(window).CSSStyleSheet;
	if (typeof Sheet !== 'function' || !('replaceSync' in Sheet.prototype)) {
		return null;
	}

	let sheet = sheetsByCss.get(css);
	if (sheet == null) {
		try {
			sheet = new Sheet() as CSSStyleSheet;
			sheet.replaceSync(css);
		} catch (e) {
			console.error(e);
			return null;
		}
		sheetsByCss.set(css, sheet);
	}
	return sheet;
}

/** The stylesheets currently adopted by a document or shadow root. */
export function adoptedSheets(root: Document | ShadowRoot): CSSStyleSheet[] {
	return Array.from(pageView(root).adoptedStyleSheets ?? []);
}

/** Replace the stylesheets adopted by a document or shadow root. */
export function setAdoptedSheets(root: Document | ShadowRoot, sheets: CSSStyleSheet[]) {
	try {
		const list = new (pageView(window).Array)();
		for (const sheet of sheets) {
			list.push(sheet);
		}
		pageView(root).adoptedStyleSheets = list;
	} catch (e) {
		console.error(e);
	}
}
