<script>
	import { LocalLinter, binary } from 'harper.js';
	import { Button } from 'flowbite-svelte';

	let linter = new LocalLinter({ binary });

	let head = `lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      linters = {
`;

	let tail = `      }
    }
  },
}`;

	async function generateConfig() {
		let default_config = await linter.getDefaultLintConfig();

		let rows = Object.entries(default_config)
			.map(([key, value]) => `\t\t\t${key} = ${value},`)
			.reduce((prev, cur) => prev + '\n' + cur);

		return head + rows + tail;
	}

	async function copyConfig() {
		let defaultConfig = await generateConfig();
		navigator.clipboard.writeText(defaultConfig);
	}
</script>

<Button onclick={copyConfig}>Copy Default Config to Clipboard</Button>
