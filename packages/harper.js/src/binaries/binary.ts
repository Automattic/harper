import { default as binaryUrl } from 'harper-wasm/harper_wasm_bg.wasm?no-inline';
import { BinaryModuleImpl } from '../BinaryModule';

/** A version of the Harper WebAssembly binary stored as its own module. */
export const binary = /*@__PURE__*/ BinaryModuleImpl.create(binaryUrl);
