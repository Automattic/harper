import { Dialect, type InitInput, type Linter as WasmLinter } from 'harper-wasm';
import { default as binaryInlinedUrl } from 'harper-wasm/harper_wasm_bg.wasm?inline';
import { default as binaryUrl } from 'harper-wasm/harper_wasm_bg.wasm?no-inline';
import LazyPromise from 'p-lazy';
import pMemoize from 'p-memoize';
import type { LintConfig } from './main';

const wasmInstances = new WeakMap<object, { memory?: WebAssembly.Memory }>();

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
	const wasm = await exports.default({ module_or_path: input });
	wasmInstances.set(exports as object, wasm as { memory?: WebAssembly.Memory });

	return exports;
});

/** A wrapper around the underlying WebAssembly module that contains Harper's core code. Used to construct a `Linter`, as well as access some miscellaneous other functions. */
export class BinaryModule {
	public url: string | URL = '';
	private inner: Promise<typeof import('harper-wasm')> | null = null;

	/** Load a binary from a specified URL. This is the only recommended way to construct this type. */
	public static create(url: string | URL): BinaryModule {
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

	public async getWasmMemoryUsageBytes(): Promise<number | undefined> {
		if (!this.inner) {
			return undefined;
		}

		const exported = await this.inner;
		const wasm = wasmInstances.get(exported as object);
		const memory = wasm?.memory;

		if (!memory?.buffer) {
			return undefined;
		}

		return memory.buffer.byteLength;
	}
}

export class SuperBinaryModule extends BinaryModule {
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

/** A version of the Harper WebAssembly binary stored inline as a data URL.
 * Can be tree-shaken if unused. */
export const binary = /*@__PURE__*/ BinaryModule.create(binaryUrl);

/** A version of the Harper WebAssembly binary stored inline as a data URL.
 * Can be tree-shaken if unused. */
export const binaryInlined = /*@__PURE__*/ BinaryModule.create(binaryInlinedUrl);
