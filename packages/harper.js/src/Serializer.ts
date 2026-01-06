import type { BinaryModule, SuperBinaryModule } from './binary';
import { assert } from './utils';

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

/** An internal class that helps the `WorkerLinter` shuffle data across a messaging channel. */
export default class Serializer {
	binary: SuperBinaryModule;

	constructor(binary: BinaryModule) {
		this.binary = binary as SuperBinaryModule;
		this.binary.setup();
	}

	async serializeArg(arg: any): Promise<RequestArg> {
		const { Lint, Span, Suggestion } = await this.binary.getBinaryModule();

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
			return {
				json: JSON.stringify(
					await Promise.all(
						Object.entries(arg).map(([key, value]) => this.serializeArg([key, value])),
					),
				),
				type: 'object',
			};
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
		const { Lint, Span, Suggestion } = await this.binary.getBinaryModule();

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
				return Object.fromEntries(
					await Promise.all(parsed.map((val: any) => this.deserializeArg(val))),
				);
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
