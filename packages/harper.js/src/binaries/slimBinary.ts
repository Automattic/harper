import { default as slimBinaryUrl } from 'harper-wasm/harper_wasm_slim_bg.wasm?no-inline';
import { BinaryModule } from '../BinaryModule';

/** A version of the slimmed-down Harper WebAssembly binary stored as its own module. */
export const slimBinary = /*@__PURE__*/ BinaryModule.create(slimBinaryUrl);
