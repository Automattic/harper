/// <reference lib="webworker" />
import LocalLinter from '../LocalLinter';
import { deserialize, serializeArg, SerializedRequest, isSerializedRequest } from './communication';

const scope = self as unknown as ServiceWorkerGlobalScope;

scope.onmessage = (e) => {
	const binary = e.data;
	if (typeof binary !== 'string') {
		throw new TypeError(`Expected binary to be a string of url but got ${typeof binary}.`);
	}
	const linter = new LocalLinter({ binary });

	async function processRequest(v: SerializedRequest) {
		const { procName, args } = await deserialize(v);

		if (procName in linter) {
			// @ts-expect-error
			const res = await linter[procName](...args);
			postMessage(await serializeArg(res));
		}
	}

	scope.onmessage = (e) => {
		isSerializedRequest(e.data) && processRequest(e.data);
	};
};

// Notify the main thread that we are ready
postMessage('ready');
