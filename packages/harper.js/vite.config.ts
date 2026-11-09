import { playwright } from '@vitest/browser-playwright';
import type { Plugin } from 'vite';
import dts from 'vite-plugin-dts';
import { defineConfig } from 'vitest/config';
import apiExtractorConfig from './api-extractor.json' with { type: 'json' };

/** Remove duplicate worker WASM assets; the public binary entries own these files. */
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
			entry: {
				index: './src/main.ts',
				binary: './src/binaries/binary.ts',
				slimBinary: './src/binaries/slimBinary.ts',
				binaryInlined: './src/binaries/binaryInlined.ts',
				slimBinaryInlined: './src/binaries/slimBinaryInlined.ts',
			},
			formats: ['es'],
		},
		minify: false,
		assetsInlineLimit: 0,
		rolldownOptions: {
			external: [/^node:/, 'fs'],
			output: {
				minifyInternalExports: false,
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
			include: ['src'],
		}),
	],
	worker: {
		format: 'es',
		plugins: () => [removeAssetsPlugin({ test: /\.wasm$/ })],
		rolldownOptions: {
			output: {
				codeSplitting: false,
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
			provider: playwright(),
			enabled: true,
			headless: true,
			screenshotFailures: false,
			instances: [{ browser: 'chromium' }, { browser: 'firefox' }],
		},
	},
	assetsInclude: ['**/*.wasm'],
});
