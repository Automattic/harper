import path from 'node:path';
import { runTests } from '@vscode/test-electron';

(async () => {
	try {
		await runTests({
			extensionDevelopmentPath: path.join(__dirname, '..', '..'),
			extensionTestsPath: path.join(__dirname, 'suite'),
			launchArgs: [
				'--disable-extensions',
				path.join(__dirname, '..', '..', 'src', 'tests', 'fixtures'),
			],
		});
	} catch (error) {
		console.error('Failed to run tests', error);
		process.exit(1);
	}
})();
