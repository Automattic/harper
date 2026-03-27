import { default as slimBinaryInlinedUrl } from 'harper-wasm/harper_wasm_slim_bg.wasm?inline';
import { BinaryModuleImpl } from '../BinaryModule';

/** A version of the slimmed-down Harper WebAssembly binary stored inline as a data URL. */
export const slimBinaryInlined = /*@__PURE__*/ BinaryModuleImpl.create(slimBinaryInlinedUrl);
