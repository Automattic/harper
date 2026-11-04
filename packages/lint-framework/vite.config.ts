import { resolve } from 'path';
import { defineConfig } from 'vite';
import dts from 'vite-plugin-dts';

export default defineConfig({
	build: {
		lib: {
			entry: resolve(import.meta.dirname, 'src/index.ts'),
			name: 'lintFramework',
			fileName: 'index',
			formats: ['es'],
		},
		minify: true,
		rolldownOptions: {
			external: ['harper.js'],
			output: {
				codeSplitting: false,
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
			include: ['src'],
		}),
	],
});
