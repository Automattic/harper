/// <reference types="vitest/config" />

import { resolve } from 'node:path';
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
						configDir: resolve(import.meta.dirname, '.storybook'),
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
			entry: { index: resolve(import.meta.dirname, 'src/lib/bundle.ts') },
			formats: ['es'],
			fileName: () => 'index',
			cssFileName: 'harper-editor',
		},
		rolldownOptions: {
			external: ['svelte', /^svelte\//, /^harper\.js(\/.*)?$/],
			output: {
				preserveModules: true,
				assetFileNames: (assetInfo) =>
					assetInfo.names.includes('harper-editor.css')
						? 'harper-editor.css'
						: 'assets/[name]-[hash][extname]',
				chunkFileNames: 'chunks/[name]-[hash].js',
				entryFileNames: '[name].js',
			},
		},
	},
});
