/// <reference lib="webworker" />
import './shims';
import { isSerializedRequest, SerializedRequest, BinaryModule } from '../binary';
import LocalLinter from '../LocalLinter';

// Notify the main thread that we are ready
self.postMessage('ready');

self.onmessage = (e) => {
	const binaryUrl = e.data;
	if (typeof binaryUrl !== 'string') {
		throw new TypeError(`Expected binary to be a string of url but got ${typeof binaryUrl}.`);
	}
	const binary = new BinaryModule(binaryUrl);
	const linter = new LocalLinter({ binary });

	async function processRequest(v: SerializedRequest) {
		const { procName, args } = await binary.deserialize(v);

		if (procName in linter) {
			// @ts-expect-error
			const res = await linter[procName](...args);
			postMessage(await binary.serializeArg(res));
		}
	}

	self.onmessage = (e) => {
		if (isSerializedRequest(e.data)) {
			processRequest(e.data);
		}
	};
};
