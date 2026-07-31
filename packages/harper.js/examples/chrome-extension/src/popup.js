// These files are copied out of the `harper.js` package by `build.js`, since
// extensions must ship all of their code locally.
import { binaryInlined } from './vendor/harper.js/binaryInlined.js';
import { Dialect, LocalLinter } from './vendor/harper.js/index.js';
import { harperVersion } from './version.js';

// Use `LocalLinter` here: `WorkerLinter` spawns its Web Worker from a blob URL,
// which the extension content security policy does not permit.
const linter = new LocalLinter({
	binary: binaryInlined,
	dialect: Dialect.American,
});

async function onInput(e) {
	const lints = await linter.lint(e.target.value);

	const list = document.getElementById('errorlist');
	list.innerHTML = '';

	for (const lint of lints) {
		const item = document.createElement('LI');
		item.appendChild(document.createTextNode(lint.message()));
		list.appendChild(item);
	}
}

const inputField = document.getElementById('maininput');
inputField.addEventListener('input', onInput);
onInput({ target: inputField });

document.getElementById('version').innerText = `harper.js ${harperVersion}`;
