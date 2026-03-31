import { default as binaryInlinedUrl } from 'harper-wasm/harper_wasm_bg.wasm?inline';
import { BinaryModuleImpl } from '../BinaryModule';

/** A version of the Harper WebAssembly binary stored inline as a data URL. */
export const binaryInlined = /*@__PURE__*/ BinaryModuleImpl.create(binaryInlinedUrl);
