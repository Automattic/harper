<script lang="ts">
import { Card } from 'flowbite-svelte';
import { type WorkerLinter } from 'harper.js';
import { LintFramework, unpackLint } from 'harper-content-framework';
import demo from '../../../../demo.md?raw';

export let content = demo;

let editor: HTMLTextAreaElement | null;
let linter: WorkerLinter;
let lfw = new LintFramework(
	async (text) => {
		// Guard until the linter is ready
		if (!linter) return [] as any;

		const raw = await linter.lint(text);
		// The framework expects "unpacked" lints with plain fields
		const unpacked = await Promise.all(
			raw.map((lint) => unpackLint(window.location.hostname, lint as any, linter as any)),
		);
		return unpacked as any;
	},
	{
		ignoreLint: async (hash: string) => {},
		getActivationKey: async () => 'off',
		openOptions: async () => {},
		addToUserDictionary: async (words: string[]) => {},
	},
);

(async () => {
	let { WorkerLinter, binary } = await import('harper.js');
	linter = new WorkerLinter({ binary });

	await linter.setup();
})();

$: if (editor != null) {
	console.log(editor);
	lfw.addTarget(editor);
}
</script>

<Card
	class="flex-grow h-full p-5 grid z-10 max-w-full text-lg overflow-auto mr-5"
>
	<textarea
		bind:this={editor}
		class="w-full text-nowrap m-0 rounded-none p-0 z-0 bg-transparent overflow-hidden border-none text-lg resize-none focus:border-0"
		bind:value={content}
	></textarea>
</Card>
