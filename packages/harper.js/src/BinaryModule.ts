import * as defaultGlue from 'harper-wasm';
import { Dialect, type InitInput, type Linter as WasmLinter } from 'harper-wasm';
import * as fullGlue from 'harper-wasm/harper_wasm.js';

import LazyPromise from 'p-lazy';
import pMemoize from 'p-memoize';
import type { LintConfig } from './main';

type WasmModule = typeof fullGlue;

function loadGlue(binary: string): WasmModule {
	if (binary.includes('harper_wasm_slim')) {
		return defaultGlue as WasmModule;
	}

	return fullGlue;
}

function getDefaultGlueBinary(binary: string): string | null {
	if (binary.includes('harper_wasm_slim')) {
		return binary;
	}

	if (binary.includes('harper_wasm_bg.wasm')) {
		return binary.replace('harper_wasm_bg.wasm', 'harper_wasm_slim_bg.wasm');
	}

	return null;
}

function getInitInput(binary: string): InitInput {
	if (typeof process !== 'undefined' && binary.startsWith('file://')) {
		return import(/* webpackIgnore: true */ /* @vite-ignore */ 'fs').then(
			(fs) =>
				new Promise<Uint8Array>((resolve, reject) => {
					fs.readFile(new URL(binary).pathname, (err, data) => {
						if (err) reject(err);
						resolve(data);
					});
				}),
		);
	}

	return binary;
}

const loadBinary = pMemoize(async (binary: string) => {
	const exports = loadGlue(binary);

	const defaultGlueBinary = getDefaultGlueBinary(binary);
	if (defaultGlueBinary != null) {
		await defaultGlue.default({ module_or_path: getInitInput(defaultGlueBinary) });
	}

	await exports.default({ module_or_path: getInitInput(binary) });

	return exports;
});

export interface BinaryModule {
	url: string | URL;

	getDefaultLintConfigAsJSON(): Promise<string>;

	getDefaultLintConfig(): Promise<LintConfig>;

	toTitleCase(text: string): Promise<string>;

	setup(): Promise<void>;
}

export function createBinaryModuleFromUrl(url: string): BinaryModule {
	return BinaryModuleImpl.create(url);
}

/** A wrapper around the underlying WebAssembly module that contains Harper's core code. Used to construct a `Linter`, as well as access some miscellaneous other functions. */
export class BinaryModuleImpl {
	public url: string | URL = '';
	private inner: Promise<WasmModule> | null = null;

	/** Load a binary from a specified URL. This is the only recommended way to construct this type. */
	public static create(url: string | URL): BinaryModuleImpl {
		const module = new SuperBinaryModule();

		module.url = url;
		module.inner = LazyPromise.from(() =>
			loadBinary(typeof module.url === 'string' ? module.url : module.url.href),
		);

		return module;
	}

	public async getDefaultLintConfigAsJSON(): Promise<string> {
		const exported = await this.inner!;
		return exported.get_default_lint_config_as_json();
	}

	public async getDefaultLintConfig(): Promise<LintConfig> {
		const exported = await this.inner!;
		return exported.get_default_lint_config();
	}

	public async toTitleCase(text: string): Promise<string> {
		const exported = await this.inner!;
		return exported.to_title_case(text);
	}

	public async setup(): Promise<void> {
		const exported = await this.inner!;
		exported.setup();
	}
}

export class SuperBinaryModule extends BinaryModuleImpl {
	async createLinter(dialect?: Dialect): Promise<WasmLinter> {
		const exported = await this.getBinaryModule();
		return exported.Linter.new(dialect ?? Dialect.American);
	}

	async getBinaryModule(): Promise<any> {
		return await LazyPromise.from(() =>
			loadBinary(typeof this.url === 'string' ? this.url : this.url.href),
		);
	}
}
