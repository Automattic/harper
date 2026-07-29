/// <reference types="vitest" />
import { playwright } from '@vitest/browser-playwright';
import { resolve } from 'path';
import { defineConfig } from 'vite';
import dts from 'vite-plugin-dts';

export default defineConfig({
	test: {
		browser: {
			provider: playwright(),
			enabled: true,
			headless: true,
			screenshotFailures: false,
			instances: [{ browser: 'chromium' }],
		},
	},
	build: {
		lib: {
			entry: resolve(__dirname, 'src/index.ts'),
			name: 'lintFramework',
			fileName: 'index',
			formats: ['es'],
		},
		minify: true,
		rollupOptions: {
			external: ['harper.js'],
			output: {
				inlineDynamicImports: true,
				minifyInternalExports: true,
			},
			treeshake: {
				moduleSideEffects: false,
				propertyReadSideEffects: false,
			},
		},
	},
	plugins: [
		dts({
			rollupTypes: true,
			tsconfigPath: './tsconfig.json',
		}),
	],
});
