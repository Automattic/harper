import { BinaryModule, LocalLinter } from 'harper.js';
import type { AppliedSuggestionResponse, ApplySuggestionRequest, LintRequest, LintResponse, Request, Response } from '../protocol';
import unpackLint from '../unpackLint';
console.log('background is running');

const linter = new LocalLinter({
	binary: new BinaryModule(chrome.runtime.getURL('./wasm/harper_wasm_bg.wasm')),
});

linter.setup();

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
	console.log(request);

	(async () => {


		sendResponse(unpackedLints);
	})();

	return true;
});

function handleRequest(message: Request): Promise<Response> {
  switch(message.kind){
      case 'lint':
        return handleLint(message);
  }
}

async function handleLint(req: LintRequest): Promise<LintResponse>{
		const lints = await linter.lint(req.text);
		const unpackedLints = lints.map(unpackLint);
    return {kind: 'lints', lints: unpackedLints};
}
