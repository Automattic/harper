import { fileURLToPath } from 'node:url';
import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', '.md'],
	preprocess: [
		vitePreprocess(),
		mdsvex({
			extensions: ['.md'],
			layout: fileURLToPath(new URL('./src/lib/docs/MarkdownLayout.svelte', import.meta.url)),
		}),
	],
	kit: {
		csrf: {
			trustedOrigins: [
				'chrome-extension://lodbfhdipoipcjmlebjbgmmgekckhpfb',
				'chrome-extension://hkjdmakdmihopipoiplebkelbhebigea',
				'chrome-extension://ihjkkjfembmnjldmdchmadigpmapkpdh',
				'moz-extension://a684f72a-270e-4cc2-a215-98cab921e95d',
			],
		},
		prerender: {
			entries: [],
		},
		adapter: adapter({
			out: 'build',
		}),
	},
};

export default config;
