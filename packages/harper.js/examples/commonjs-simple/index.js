async function main() {
	const harper = await import('harper.js');
	// We cannot use `WorkerLinter` on Node.js since it relies on web-specific APIs.
	// This constructs the linter to consume American English.
	const linter = new harper.LocalLinter({
		binary: harper.binary,
		dialect: harper.Dialect.American,
	});

	const lints = await linter.lint('This is a example of how to use `harper.js`.');

	console.log('Here are the results of linting the above text:');

	for (const lint of lints) {
		console.log(' - ', lint.span().start, ':', lint.span().end, lint.message());

		if (lint.suggestion_count() !== 0) {
			console.log('Suggestions:');

			for (const sug of lint.suggestions()) {
				console.log(
					'\t - ',
					sug.kind() === 1 ? 'Remove' : 'Replace with',
					sug.get_replacement_text(),
				);
			}
		}
	}
}

main();
