import external from 'rollup-plugin-peer-deps-external';
import svg from 'rollup-plugin-svg-import';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [svg({ stringify: true }), external()],
	build: {
		outDir: '.',
		target: 'es6',
		lib: {
			entry: 'src/index.ts',
			formats: ['cjs'],
			fileName: 'main',
		},
		rollupOptions: {
			external: ['obsidian', 'electron'],
			output: {
				inlineDynamicImports: true,
			},
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
});
