/// <reference types="vitest/config" />

import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { playwright } from '@vitest/browser-playwright';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), svelte()],
	test: {
		projects: [
			{
				extends: true,
				plugins: [
					storybookTest({
						configDir: resolve(dirname(fileURLToPath(import.meta.url)), '.storybook'),
					}),
				],
				test: {
					name: 'storybook',
					browser: {
						enabled: true,
						headless: true,
						provider: playwright({}),
						instances: [{ browser: 'chromium' }],
					},
				},
			},
		],
	},
	build: {
		lib: {
			entry: resolve(__dirname, 'src/lib/bundle.ts'),
			formats: ['es'],
			fileName: () => 'index',
			cssFileName: 'harper-editor',
		},
		rollupOptions: {
			external: ['svelte', /^svelte\//, /^harper\.js(\/.*)?$/],
			output: {
				assetFileNames: (assetInfo) =>
					assetInfo.name === 'harper-editor.css'
						? 'harper-editor.css'
						: 'assets/[name]-[hash][extname]',
				chunkFileNames: 'chunks/[name]-[hash].js',
				entryFileNames: () => 'index.js',
			},
		},
	},
});
