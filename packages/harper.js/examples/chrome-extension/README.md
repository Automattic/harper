# Chrome extension popup

Chrome extensions cannot load executable code from a CDN, so bundle `harper.js`
with your extension instead of importing it from `unpkg` at runtime. This
minimal Manifest V3 popup shows the pieces you need.

Install the package in your extension project:

```sh
npm install harper.js
```

Use a popup page that loads your bundled script:

```json
{
  "manifest_version": 3,
  "name": "Harper popup example",
  "version": "0.0.1",
  "action": {
    "default_popup": "popup.html"
  }
}
```

```html
<textarea id="source">This is an test.</textarea>
<pre id="output">Loading Harper...</pre>
<script type="module" src="./popup.js"></script>
```

Then import Harper from npm in the source file that your bundler emits as
`popup.js`:

```ts
import { WorkerLinter } from 'harper.js';
import { binaryInlined } from 'harper.js/binaryInlined';

const linter = new WorkerLinter({ binary: binaryInlined });

const source = document.querySelector<HTMLTextAreaElement>('#source');
const output = document.querySelector<HTMLElement>('#output');

async function update() {
  if (!source || !output) {
    return;
  }

  const lints = await linter.lint(source.value, { language: 'plaintext' });
  output.textContent =
    lints.length === 0
      ? 'No suggestions.'
      : lints.map((lint) => lint.message()).join('\n');
}

source?.addEventListener('input', update);
void update();
```

The `binaryInlined` entry point embeds the WebAssembly binary in the bundled
JavaScript, which avoids extra extension asset paths while you are getting
started. If you prefer to ship the Wasm file separately, import `binary` from
`harper.js/binary` and make sure your bundler copies the generated Wasm asset
into the extension package.
