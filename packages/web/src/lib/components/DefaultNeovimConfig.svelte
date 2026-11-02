<script>
import { Button } from 'components';

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
	// This function runs only from the browser's copy-button handler.
	const { binary } = await import('harper.js/binary');
	let default_config = await binary.getDefaultLintConfig();

	let rows = Object.entries(default_config)
		.map(([key, value]) => `\t\t\t${key} = ${value},`)
		.reduce((prev, cur) => `${prev}\n${cur}`);

	return head + rows + tail;
}

async function copyConfig() {
	let defaultConfig = await generateConfig();
	navigator.clipboard.writeText(defaultConfig);
}
</script>

<Button onclick={copyConfig}>Copy Default Config to Clipboard</Button>
