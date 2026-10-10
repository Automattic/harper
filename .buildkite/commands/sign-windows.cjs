const { execFileSync } = require('node:child_process');

// Tauri calls this for the app, NSIS uninstaller, installer, and bundled DLLs.
// setup_azure_trusted_signing.ps1 supplies these paths and Azure credentials.
function signWindows(file, env = process.env, run = execFileSync) {
	if (!file || !file.trim()) {
		throw new Error('Expected a file to sign.');
	}

	const required = ['SIGNTOOL_PATH', 'AZURE_CODE_SIGNING_DLIB', 'AZURE_METADATA_JSON'];
	const missing = required.filter((name) => !env[name]?.trim());
	if (missing.length) {
		throw new Error(`Azure signing setup is incomplete: ${missing.join(', ')}`);
	}

	run(
		env.SIGNTOOL_PATH,
		[
			'sign',
			'/v',
			'/fd',
			'SHA256',
			'/tr',
			env.AZURE_TIMESTAMP_SERVER || 'http://timestamp.acs.microsoft.com',
			'/td',
			'SHA256',
			'/dlib',
			env.AZURE_CODE_SIGNING_DLIB,
			'/dmdf',
			env.AZURE_METADATA_JSON,
			file,
		],
		{ stdio: 'inherit' },
	);
	// Verify before Tauri embeds this file in another executable. The bundler
	// restores the original app binary after packaging, so checking it later
	// would not check the executable that users actually install.
	run(env.SIGNTOOL_PATH, ['verify', '/pa', '/all', '/v', '/tw', file], { stdio: 'inherit' });
}

if (require.main === module) {
	if (process.argv.length !== 3) {
		throw new Error('Usage: node sign-windows.cjs <file>');
	}
	signWindows(process.argv[2]);
}

module.exports = { signWindows };
