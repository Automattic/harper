import { defineManifest } from '@crxjs/vite-plugin';
import packageData from '../package.json';

//@ts-expect-error
const isDev = process.env.NODE_ENV == 'development';

/**
 * Builds a CSP string that:
 *   • always meets the MV3 minimum (`'self' 'wasm-unsafe-eval'`)
 *   • whitelists the Vite HMR server only when `isDev` is true
 *
 * NOTE: `'unsafe-eval'` is *omitted* because Chrome blocks it outright.
 */
export function makeExtensionCSP(isDev: boolean): string {
	const scriptSrc = ["'self'", "'wasm-unsafe-eval'"]; // minimum, cannot add more
	const objectSrc = ["'self'"]; // standard
	// Keep connect-src minimal: only production endpoints and origins the extension explicitly needs.
	// Add dev hosts only when isDev is true (below).
	const connectSrc = ["'self'", 'https://writewithharper.com', 'https://api.writewithharper.com'];
	// Avoid 'unsafe-inline' for style-src. Move inline styles into external CSS files or apply CSP nonce strategy if needed.
	const styleSrc = ["'self'", 'https://fonts.googleapis.com'];
	const fontSrc = ["'self'", 'https://fonts.gstatic.com', 'data:'];
 
	if (isDev) {
		// `ws://` and `http://` use the same host:port → list both
		// Allow local dev HMR and local hosts only in dev builds.
		connectSrc.push('http://localhost:5173', 'ws://localhost:5173');
		connectSrc.push('http://127.0.0.1:*', 'ws://127.0.0.1:*');
		// If you must use inline styles in development, add them only in dev and prefer external CSS for production.
		// Note: we intentionally do not add 'unsafe-inline' here to keep dev behavior closer to production.
	}

  // Production writewithharper domains already included above; keep connectSrc minimal.
  
	// Assemble the semicolon-delimited CSP
	return `${[
		`script-src ${scriptSrc.join(' ')}`,
		`object-src ${objectSrc.join(' ')}`,
		`connect-src ${connectSrc.join(' ')}`,
		`style-src ${styleSrc.join(' ')}`,
		`font-src ${fontSrc.join(' ')}`,
	].join('; ')};`;
}

export default defineManifest({
	name: `Private Grammar Checker - Harper${isDev ? ' ➡️ Dev' : ''}`,
	description: packageData.description,
	version: packageData.version,
	manifest_version: 3,
	action: {
		default_popup: 'popup.html',
	},
	options_page: 'options.html',
	browser_specific_settings: {
		gecko: {
			id: 'harper@writewithharper.com',
			strict_min_version: '146.0',
		},
	},
	background: {
		service_worker: 'src/background/index.ts',
		scripts: ['src/background/index.ts'],
		type: 'module',
	},
	content_scripts: [
		{
			matches: ['https://docs.google.com/document/*'],
			all_frames: false,
			js: ['src/contentScript/googleDocsBootstrap.js'],
			run_at: 'document_start',
			world: 'MAIN',
		},
		{
			matches: ['<all_urls>'],
			all_frames: true,
			match_about_blank: true,
			match_origin_as_fallback: true,
			js: ['src/contentScript/index.ts'],
			run_at: 'document_idle',
		},
	],
	web_accessible_resources: [
		{
			matches: ['<all_urls>'],
			resources: [
				'wasm/harper_wasm_bg.wasm',
				'google-docs-bridge.js',
				'google-docs-protocol.js',
				'google-docs-bridge-request-handler.js',
			],
		},
	],
	icons: {
		'512': 'logo.png',
	},
	permissions: ['storage', 'tabs'],
	content_security_policy: {
		extension_pages: makeExtensionCSP(isDev),
	},
	host_permissions: ['https://writewithharper.com/*'],
});
