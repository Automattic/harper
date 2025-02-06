/// <reference types="vitest" />
import { resolve } from 'path';
import dts from 'vite-plugin-dts';
import { defineConfig } from 'vite';

export default defineConfig({
	build: {
		lib: {
			entry: resolve(__dirname, 'src/main.ts'),
			fileName: `harper`,
			name: 'harper',
			formats: ['es'],
		},
		rollupOptions: {
			output: {
				minifyInternalExports: false,
				inlineDynamicImports: true
			}
		}
	},
	base: './',
	plugins: [
		dts({
			...require('./api-extractor.json'),
			rollupTypes: true,
			tsconfigPath: './tsconfig.json',
		}),
	],
	worker: {
		format: 'es',
		rollupOptions: {
			output: {
				inlineDynamicImports: true
			}
		}
	},
	server: {
		fs: {
			allow: ['../../harper-wasm/pkg']
		}
	},
	test: {
		browser: {
			provider: 'playwright',
			enabled: true,
			headless: true,
			screenshotFailures: false,
			instances: [
				{ browser: 'chromium' },
				{ browser: 'firefox' }
			]
		}
	},
	assetsInclude: ['**/*.wasm']
});
