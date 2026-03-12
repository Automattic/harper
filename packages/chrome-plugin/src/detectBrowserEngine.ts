export default function detectBrowserEngine() {
	// @ts-expect-error
	if (typeof InstallTrigger !== 'undefined') return 'firefox';
	if (typeof window.chrome !== 'undefined') return 'chromium';
	return 'unknown';
}
