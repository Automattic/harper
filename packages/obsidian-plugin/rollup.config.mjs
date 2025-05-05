import json from '@rollup/plugin-json';
import { nodeResolve } from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import { wasm } from '@rollup/plugin-wasm';
import external from 'rollup-plugin-peer-deps-external';
import svg from 'rollup-plugin-svg-import';

export default {
	input: 'src/index.ts',
	output: {
		file: 'main.js',
		format: 'cjs',
	},
	external: ['obsidian', 'electron'],
	plugins: [
		json(),
		svg({
			stringify: true,
		}),
		external(),
		wasm({ maxFileSize: 2 ** 32, publicPath: './' }),
		nodeResolve(),
		typescript({ compilerOptions: { lib: ['es5', 'es6', 'dom'], target: 'es5' } }),
	],
};
