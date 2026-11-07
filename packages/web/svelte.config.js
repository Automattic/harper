import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';
import mdsvexConfig from './mdsvex.config.js';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', '.md'],
	preprocess: [vitePreprocess(), mdsvex(mdsvexConfig)],
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
