import { defineManifest } from '@crxjs/vite-plugin';
import packageData from '../package.json';

//@ts-ignore
const isDev = process.env.NODE_ENV == 'development';

export default defineManifest({
	name: `Private Grammar Checking - Harper${isDev ? ' ➡️ Dev' : ''}`,
	description: packageData.description,
	version: packageData.version,
	manifest_version: 3,
	action: {
		default_popup: 'popup.html',
	},
	options_page: 'options.html',
	background: {
		service_worker: 'src/background/index.ts',
		type: 'module',
	},
	content_scripts: [
		{
			matches: ['http://*/*', 'https://*/*'],
			js: ['src/contentScript/index.ts'],
		},
	],
	web_accessible_resources: [
		{
			resources: [],
			matches: [],
		},
	],
	icons: {
		'512': 'logo.png',
	},
	permissions: ['storage', 'tabs'],
	content_security_policy: {
		extension_pages: "script-src 'self' 'wasm-unsafe-eval'",
	},
});
