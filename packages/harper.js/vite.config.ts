/// <reference types="vitest" />
import { resolve } from 'path';
import { defineConfig, type Plugin } from 'vite';
import dts from 'vite-plugin-dts';
import apiExtractorConfig from './api-extractor.json';

function removeAssetsPlugin(options: { test: RegExp }): Plugin {
	return {
		name: 'remove-wasm',
		generateBundle(_, bundle) {
			for (const file in bundle) {
				if (options.test.test(file)) {
					delete bundle[file];
				}
			}
		},
	};
}

export default defineConfig({
	build: {
		lib: {
			entry: resolve(__dirname, 'src/main.ts'),
			fileName: 'harper',
			name: 'harper',
			formats: ['es'],
		},
		minify: false,
		assetsInlineLimit: 0,
		rollupOptions: {
			external: [/^node:/, 'fs'],
			output: {
				minifyInternalExports: false,
				inlineDynamicImports: true,
			},
			treeshake: {
				moduleSideEffects: false,
				propertyReadSideEffects: false,
			},
		},
	},
	base: './',
	plugins: [
		dts({
			...apiExtractorConfig,
			rollupTypes: true,
			tsconfigPath: './tsconfig.json',
		}),
	],
	worker: {
		format: 'es',
		plugins: () => [removeAssetsPlugin({ test: /\.wasm$/ })],
		rollupOptions: {
			output: {
				inlineDynamicImports: true,
			},
		},
	},
	server: {
		fs: {
			allow: ['../../harper-wasm/pkg'],
		},
	},
	test: {
		retry: process.env.CI ? 5 : 0,
		browser: {
			provider: 'playwright',
			enabled: true,
			headless: true,
			screenshotFailures: false,
			instances: [{ browser: 'chromium' }],
		},
	},
	assetsInclude: ['**/*.wasm'],
});
