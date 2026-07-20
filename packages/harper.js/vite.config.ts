/// <reference types="vitest" />
import { playwright } from '@vitest/browser-playwright';
import { readFileSync } from 'fs';
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

// Plugin to resolve harper-wasm wasm file imports
function harperWasmPlugin(): Plugin {
	const harperWasmPkgPath = resolve(__dirname, '../../../harper-wasm/pkg');
	return {
		name: 'harper-wasm-resolver',
		enforce: 'pre',
		resolveId(source, _importer) {
			// Handle imports like 'harper-wasm/harper_wasm_slim_bg.wasm?inline'
			if (source.includes('harper-wasm/') && source.includes('.wasm')) {
				// Strip the query parameters for file resolution
				const filePath = source.split('?')[0];
				const wasmFile = resolve(harperWasmPkgPath, filePath.replace('harper-wasm/', ''));

				// Check if file exists
				try {
					if (readFileSync(wasmFile)) {
						// Return the resolved path, keeping the query
						const query = source.includes('?') ? `?${source.split('?')[1]}` : '';
						return { id: wasmFile + query };
					}
				} catch {
					// File doesn't exist, let other plugins handle it
				}
			}
			return null;
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
		rollupOptions: {
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
		harperWasmPlugin(),
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
			provider: playwright(),
			enabled: true,
			headless: true,
			screenshotFailures: false,
			instances: [{ browser: 'chromium' }, { browser: 'firefox' }],
		},
	},
	assetsInclude: ['**/*.wasm'],
});
