import { default as _binary } from 'harper-wasm/harper_wasm_bg.wasm?url';
import { default as _inlinedBinary } from 'harper-wasm/harper_wasm_bg.wasm?inline';

export async function loadBinary(binary: string) {
	const exports = await import('harper-wasm');
	await exports.default({ module_or_path: binary });
	return exports;
}

export const binary: string = _binary;
export const inlinedBinary: string = _inlinedBinary;
