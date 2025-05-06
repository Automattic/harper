import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', '.md'],
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter({
			out: 'build',
		}),
	},
};

export default config;
