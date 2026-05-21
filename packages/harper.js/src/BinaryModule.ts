import * as defaultGlue from 'harper-wasm';
import { Dialect, type InitInput, type Linter as WasmLinter } from 'harper-wasm';
import * as fullGlue from 'harper-wasm/harper_wasm.js';

import LazyPromise from 'p-lazy';
import pMemoize from 'p-memoize';
import type { LintConfig } from './main';

type WasmModule = typeof fullGlue;
// Adding a new flavor requires importing its generated glue and updating loadGlue.
export type WasmGlueFlavor = 'full' | 'slim';

function inferGlueFlavor(binary: string): WasmGlueFlavor {
	return binary.includes('harper_wasm_slim') ? 'slim' : 'full';
}

/**
 * Resolve the glue flavor for a binary module.
 *
 * `glueFlavor` is intentionally optional on the public `BinaryModule` interface so
 * older/custom BinaryModule-like objects remain structurally compatible. When it is
 * absent, fall back to inferring the flavor from the binary URL.
 */
export function resolveWasmGlueFlavor(binary: Pick<BinaryModule, 'url' | 'glueFlavor'>): WasmGlueFlavor {
	return binary.glueFlavor ?? inferGlueFlavor(typeof binary.url === 'string' ? binary.url : binary.url.href);
}

function loadGlue(glueFlavor: WasmGlueFlavor): WasmModule {
	if (glueFlavor === 'slim') {
		return defaultGlue as WasmModule;
	}

	return fullGlue;
}

function getDefaultGlueBinary(binary: string, glueFlavor: WasmGlueFlavor): string | null {
	if (glueFlavor === 'slim') {
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

const loadBinaryWithKey = pMemoize(
	async (_cacheKey: string, binary: string, glueFlavor: WasmGlueFlavor) => {
		const exports = loadGlue(glueFlavor);

		const defaultGlueBinary = getDefaultGlueBinary(binary, glueFlavor);
		if (defaultGlueBinary != null) {
			try {
				await defaultGlue.default({ module_or_path: getInitInput(defaultGlueBinary) });
			} catch (err) {
				if (glueFlavor === 'slim') {
					throw err;
				}
			}
		}

		await exports.default({ module_or_path: getInitInput(binary) });

		return exports;
	},
);

function loadBinary(binary: string, glueFlavor: WasmGlueFlavor) {
	return loadBinaryWithKey(`${glueFlavor}:${binary}`, binary, glueFlavor);
}

export interface BinaryModule {
	url: string | URL;
	glueFlavor?: WasmGlueFlavor;

	getDefaultLintConfigAsJSON(): Promise<string>;

	getDefaultLintConfig(): Promise<LintConfig>;

	toTitleCase(text: string): Promise<string>;

	setup(): Promise<void>;
}

export function createBinaryModuleFromUrl(url: string, glueFlavor?: WasmGlueFlavor): BinaryModule {
	return BinaryModuleImpl.create(url, glueFlavor);
}

/** A wrapper around the underlying WebAssembly module that contains Harper's core code. Used to construct a `Linter`, as well as access some miscellaneous other functions. */
export class BinaryModuleImpl {
	public url: string | URL = '';
	public glueFlavor: WasmGlueFlavor = 'full';
	private inner: Promise<WasmModule> | null = null;

	/** Load a binary from a specified URL. This is the only recommended way to construct this type. */
	public static create(url: string | URL, glueFlavor?: WasmGlueFlavor): BinaryModuleImpl {
		const module = new SuperBinaryModule();

		module.url = url;
		module.glueFlavor = glueFlavor ?? inferGlueFlavor(typeof url === 'string' ? url : url.href);
		module.inner = LazyPromise.from(() =>
			loadBinary(typeof module.url === 'string' ? module.url : module.url.href, module.glueFlavor),
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
			loadBinary(typeof this.url === 'string' ? this.url : this.url.href, this.glueFlavor),
		);
	}
}
