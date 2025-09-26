import { defineManifest } from '@crxjs/vite-plugin';
import packageData from '../package.json';

//@ts-ignore
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
	const connectSrc = ["'self'"]; // WebSocket goes here

	if (isDev) {
		// `ws://` and `http://` use the same host:port → list both
		connectSrc.push('http://localhost:5173', 'ws://localhost:5173');
		// include the 127.0.0.1 loopback in case you switch hosts
		connectSrc.push('http://127.0.0.1:*', 'ws://127.0.0.1:*');
	}

	// Assemble the semicolon-delimited CSP
	return `${[
		`script-src ${scriptSrc.join(' ')}`,
		`object-src ${objectSrc.join(' ')}`,
		`connect-src ${connectSrc.join(' ')}`,
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
			strict_min_version: '135.0',
		},
	},
	background: {
		service_worker: 'src/background/index.ts',
		scripts: ['src/background/index.ts'],
		type: 'module',
	},
	content_scripts: [
		{
			matches: ['<all_urls>'],
			all_frames: true,
			match_about_blank: true,
			js: ['src/contentScript/index.ts'],
			run_at: 'document_idle',
		},
	],
	web_accessible_resources: [
		{
			matches: ['<all_urls>'],
			resources: ['wasm/harper_wasm_bg.wasm'],
		},
	],
	icons: {
		'512': 'logo.png',
	},
	permissions: ['storage', 'tabs'],
	content_security_policy: {
		extension_pages: makeExtensionCSP(isDev),
	},
});
