import { default as binaryUrl } from 'harper-wasm/harper_wasm_bg.wasm?no-inline';
import { default as binaryInlinedUrl } from 'harper-wasm/harper_wasm_bg.wasm?inline';
import type { Span, Suggestion, Linter as WasmLinter } from 'harper-wasm';
import { invariant } from './utils';
import { LintConfig } from './main';

const _loadedBinaryMap = new Map<string, typeof import('harper-wasm')>();

export async function loadBinary(binary: string): Promise<typeof import('harper-wasm')> {
	if (_loadedBinaryMap.has(binary)) {
		return _loadedBinaryMap.get(binary)!;
	} else {
		const exports = await import('harper-wasm');
		await exports.default({ module_or_path: binary });
		_loadedBinaryMap.set(binary, exports);
		return exports;
	}
}

export type SerializableTypes =
	| 'string'
	| 'number'
	| 'boolean'
	| 'Suggestion'
	| 'Lint'
	| 'Span'
	| 'Array'
	| 'undefined';

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
	private exported: Promise<typeof import('harper-wasm')>;

	constructor(public url: string | URL) {
		this.exported = import('harper-wasm').then(async (exports) => {
			await exports.default({ module_or_path: url });
			return exports;
		});
	}

	async applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string> {
		const exported = await this.exported;
		return exported.apply_suggestion(text, span, suggestion);
	}

	async getDefaultLintConfigAsJSON(): Promise<string> {
		const exported = await this.exported;
		return exported.get_default_lint_config_as_json();
	}

	async getDefaultLintConfig(): Promise<LintConfig> {
		const exported = await this.exported;
		return exported.get_default_lint_config();
	}

	async toTitleCase(text: string): Promise<string> {
		const exported = await this.exported;
		return exported.to_title_case(text);
	}

	async setup(): Promise<void> {
		const exported = await this.exported;
		exported.setup();
	}

	async createLinter(): Promise<WasmLinter> {
		const exported = await this.exported;
		return exported.Linter.new();
	}

	async serializeArg(arg: any): Promise<RequestArg> {
		const { Lint, Span, Suggestion } = await this.exported;

		if (Array.isArray(arg)) {
			return {
				json: JSON.stringify(await Promise.all(arg.map((a) => this.serializeArg(a)))),
				type: 'Array'
			};
		}

		const argType = typeof arg;
		switch (argType) {
			case 'string':
			case 'number':
			case 'boolean':
			case 'undefined':
				return { json: JSON.stringify(arg), type: argType };
		}

		if (arg.to_json != undefined) {
			const json = arg.to_json();
			let type: SerializableTypes | undefined = undefined;

			if (arg instanceof Lint) {
				type = 'Lint';
			} else if (arg instanceof Suggestion) {
				type = 'Suggestion';
			} else if (arg instanceof Span) {
				type = 'Span';
			}

			if (type == undefined) {
				throw new Error('Unhandled case');
			}

			return { json, type };
		}

		throw new Error('Unhandled case');
	}

	async serialize(req: DeserializedRequest): Promise<SerializedRequest> {
		return {
			procName: req.procName,
			args: await Promise.all(req.args.map((arg) => this.serializeArg(arg)))
		};
	}

	async deserializeArg(requestArg: RequestArg): Promise<any> {
		const { Lint, Span, Suggestion } = await this.exported;

		switch (requestArg.type) {
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
				invariant(Array.isArray(parsed));
				return await Promise.all(parsed.map((arg) => this.deserializeArg(arg)));
			}
			default:
				throw new Error(`Unhandled case: ${requestArg.type}`);
		}
	}

	async deserialize(request: SerializedRequest): Promise<DeserializedRequest> {
		return {
			procName: request.procName,
			args: await Promise.all(request.args.map((arg) => this.deserializeArg(arg)))
		};
	}
}

export const binary = /*@__PURE__*/ new BinaryModule(binaryUrl);

export const binaryInlined = /*@__PURE__*/ new BinaryModule(binaryInlinedUrl);
