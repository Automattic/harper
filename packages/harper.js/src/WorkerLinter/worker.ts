/// <reference lib="webworker" />
import './shims';
import { SuperBinaryModule, type WasmGlueFlavor } from '../BinaryModule';
import LocalLinter from '../LocalLinter';
import Serializer, { isSerializedRequest, type SerializedRequest } from '../Serializer';

// Notify the main thread that we are ready
self.postMessage('ready');

self.onmessage = (e) => {
	const [binaryUrl, dialect, glueFlavor] = e.data;
	if (typeof binaryUrl !== 'string') {
		throw new TypeError(`Expected binary to be a string of url but got ${typeof binaryUrl}.`);
	}
	if (glueFlavor !== undefined && glueFlavor !== 'full' && glueFlavor !== 'slim') {
		throw new TypeError(`Expected glue flavor to be "full" or "slim" but got ${glueFlavor}.`);
	}
	const binary = SuperBinaryModule.create(binaryUrl, glueFlavor as WasmGlueFlavor | undefined);
	const serializer = new Serializer(binary);
	const linter = new LocalLinter({ binary, dialect });

	async function processRequest(v: SerializedRequest) {
		const { procName, args } = await serializer.deserialize(v);

		if (procName in linter) {
			// @ts-expect-error
			const res = await linter[procName](...args);
			postMessage(await serializer.serializeArg(res));
		}
	}

	self.onmessage = (e) => {
		if (isSerializedRequest(e.data)) {
			processRequest(e.data);
		}
	};
};
