import { defineConfig } from 'vite';
import svg from 'rollup-plugin-svg-import';
import external from 'rollup-plugin-peer-deps-external';

export default defineConfig({
	plugins: [
		svg({ stringify: true }),
		external(),
	],
	build: {
    outDir: ".",
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
});
