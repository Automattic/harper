import { crx } from '@crxjs/vite-plugin';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';
import copy from 'rollup-plugin-copy';
import sveltePreprocess from 'svelte-preprocess';
import { defineConfig, loadEnv } from 'vite';
import manifest from './src/manifest';

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, process.cwd(), '');

	const browser = env.TARGET_BROWSER ?? 'chrome';

	if (!['chrome', 'firefox'].includes(browser)) {
		throw new Error('UNSUPPORTED BROWSER TYPE');
	}

	console.log(`Building for ${browser}`);

	const production = mode === 'production';

	return {
		build: {
			minify: false,
			outDir: 'build',
			rollupOptions: {
				output: {
					chunkFileNames: 'assets/chunk-[hash].js',
				},
			},
		},
		plugins: [
			copy({
				hook: 'buildStart',
				targets: [
					{
						src: '../harper.js/dist/harper_wasm_bg.wasm',
						dest: './public/wasm',
					},
				],
			}),
			tailwindcss(),
			crx({ manifest, browser }),
			svelte({
				compilerOptions: {
					dev: !production,
				},
				preprocess: sveltePreprocess(),
			}),
		],
		resolve: {
			alias: {
				'@': path.resolve(__dirname, 'src'),
			},
		},
		legacy: {
			skipWebSocketTokenCheck: true,
		},
	};
});
