/** Does a rough estimate of whether the current page is a Substack page. */
export default function isSubstack(): boolean {
	const hostname = window.location.hostname.toLowerCase();

	if (hostname === 'substack.com' || hostname.endsWith('.substack.com')) {
		return true;
	}

	if (document.querySelector('link[rel="preconnect"][href*="substackcdn.com"]')) {
		return true;
	}

	if (document.querySelector('meta[property="og:title"][content*="| Substack"]')) {
		return true;
	}

	if (document.querySelector('meta[name="description"][content*="a Substack publication"]')) {
		return true;
	}

	return false;
}
