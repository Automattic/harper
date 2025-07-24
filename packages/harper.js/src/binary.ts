import { Dialect, type InitInput, type Linter as WasmLinter } from 'harper-wasm';
import { default as binaryInlinedUrl } from 'harper-wasm/harper_wasm_bg.wasm?inline';
import { default as binaryUrl } from 'harper-wasm/harper_wasm_bg.wasm?no-inline';
import LazyPromise from 'p-lazy';
import pMemoize from 'p-memoize';
import type { LintConfig } from './main';
import { assert } from './utils';

export const loadBinary = pMemoize(async (binary: string) => {
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

export type SerializableTypes =
	| 'string'
	| 'number'
	| 'boolean'
	| 'object'
	| 'Suggestion'
	| 'Lint'
	| 'Span'
	| 'Array'
	| 'undefined'
	| 'bigint';

/** Serializable argument to a procedure to be run on the web worker. */
export interface RequestArg {
	json: string;
	type: SerializableTypes;
}

/** An object that is sent to the web worker to request work to be done. */
export interface SerializedRequest {
	/** The procedure to be executed. */
	procName: string;
	/** The arguments to the procedure */
	args: RequestArg[];
}

/** An object that is received by the web worker to request work to be done. */
export interface DeserializedRequest {
	/** The procedure to be executed. */
	procName: string;
	/** The arguments to the procedure */
	args: any[];
}

export function isSerializedRequest(v: unknown): v is SerializedRequest {
	return typeof v === 'object' && v !== null && 'procName' in v && 'args' in v;
}

/** This class aims to define the communication protocol between the main thread and the worker.
 * Note that much of the complication here comes from the fact that we can't serialize function calls or referenced WebAssembly memory.*/
export class BinaryModule {
	public url: string | URL;

	private inner: Promise<typeof import('harper-wasm')>;

	constructor(url: string | URL) {
		this.url = url;
		this.inner = LazyPromise.from(() =>
			loadBinary(typeof this.url === 'string' ? this.url : this.url.href),
		);
	}

	async getDefaultLintConfigAsJSON(): Promise<string> {
		const exported = await this.inner;
		return exported.get_default_lint_config_as_json();
	}

	async getDefaultLintConfig(): Promise<LintConfig> {
		const exported = await this.inner;
		return exported.get_default_lint_config();
	}

	async toTitleCase(text: string): Promise<string> {
		const exported = await this.inner;
		return exported.to_title_case(text);
	}

	async setup(): Promise<void> {
		const exported = await this.inner;
		exported.setup();
	}

	async createLinter(dialect?: Dialect): Promise<WasmLinter> {
		const exported = await this.inner;
		return exported.Linter.new(dialect ?? Dialect.American);
	}

	async serializeArg(arg: any): Promise<RequestArg> {
		const { Lint, Span, Suggestion } = await this.inner;

		if (Array.isArray(arg)) {
			return {
				json: JSON.stringify(await Promise.all(arg.map((a) => this.serializeArg(a)))),
				type: 'Array',
			};
		}

		const argType = typeof arg;
		switch (argType) {
			case 'string':
			case 'number':
			case 'boolean':
			case 'undefined':
				return { json: JSON.stringify(arg), type: argType };
			case 'bigint':
				return { json: arg.toString(), type: argType };
		}

		if (arg.to_json !== undefined) {
			const json = arg.to_json();
			let type: SerializableTypes | undefined;

			if (arg instanceof Lint) {
				type = 'Lint';
			} else if (arg instanceof Suggestion) {
				type = 'Suggestion';
			} else if (arg instanceof Span) {
				type = 'Span';
			}

			if (type === undefined) {
				throw new Error('Unhandled case: type undefined');
			}

			return { json, type };
		}

		if (argType == 'object') {
			return { json: JSON.stringify(arg), type: 'object' };
		}

		throw new Error(`Unhandled case: ${arg}`);
	}

	async serialize(req: DeserializedRequest): Promise<SerializedRequest> {
		return {
			procName: req.procName,
			args: await Promise.all(req.args.map((arg) => this.serializeArg(arg))),
		};
	}

	async deserializeArg(requestArg: RequestArg): Promise<any> {
		const { Lint, Span, Suggestion } = await this.inner;

		switch (requestArg.type) {
			case 'bigint':
				return BigInt(requestArg.json);
			case 'undefined':
				return undefined;
			case 'boolean':
			case 'number':
			case 'string':
				return JSON.parse(requestArg.json);
			case 'Suggestion':
				return Suggestion.from_json(requestArg.json);
			case 'Lint':
				return Lint.from_json(requestArg.json);
			case 'Span':
				return Span.from_json(requestArg.json);
			case 'Array': {
				const parsed = JSON.parse(requestArg.json);
				assert(Array.isArray(parsed));
				return await Promise.all(parsed.map((arg) => this.deserializeArg(arg)));
			}
			case 'object': {
				const parsed = JSON.parse(requestArg.json);
				return parsed;
			}
			default:
				throw new Error(`Unhandled case: ${requestArg.type}`);
		}
	}

	async deserialize(request: SerializedRequest): Promise<DeserializedRequest> {
		return {
			procName: request.procName,
			args: await Promise.all(request.args.map((arg) => this.deserializeArg(arg))),
		};
	}
}

export const binary = /*@__PURE__*/ new BinaryModule(binaryUrl);

export const binaryInlined = /*@__PURE__*/ new BinaryModule(binaryInlinedUrl);
