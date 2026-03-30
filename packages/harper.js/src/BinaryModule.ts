import { Dialect, type InitInput, type Linter as WasmLinter } from 'harper-wasm';

import LazyPromise from 'p-lazy';
import pMemoize from 'p-memoize';
import type { LintConfig } from './main';

const loadBinary = pMemoize(async (binary: string) => {
	const exports = await import('harper-wasm');

	let input: InitInput;
	if (typeof process !== 'undefined' && binary.startsWith('file://')) {
		const fs = await import(/* webpackIgnore: true */ /* @vite-ignore */ 'fs');
		input = new Promise((resolve, reject) => {
			fs.readFile(new URL(binary).pathname, (err, data) => {
				if (err) reject(err);
				resolve(data);
			});
		});
	} else {
		input = binary;
	}
	await exports.default({ module_or_path: input });

	return exports;
});

export interface BinaryModule {
	url: string | URL;

	getDefaultLintConfigAsJSON(): Promise<string>;

	getDefaultLintConfig(): Promise<LintConfig>;

	toTitleCase(text: string): Promise<string>;

	setup(): Promise<void>;
}

export function createBinaryModuleFromUrl(url: string): BinaryModule{
   return BinaryModuleImpl.create(url);
}

/** A wrapper around the underlying WebAssembly module that contains Harper's core code. Used to construct a `Linter`, as well as access some miscellaneous other functions. */
export class BinaryModuleImpl {
	public url: string | URL = '';
	private inner: Promise<typeof import('harper-wasm')> | null = null;

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
