/** Does a rough estimate of whether the current page is a WordPress page. */
export default function isWordPress(): boolean {
	if (document.querySelector('meta[name="generator"][content*="WordPress"]')) {
		return true;
	}

	if (document.querySelector('link[rel="https://api.w.org/"][href]')) {
		return true;
	}

	return false;
}
