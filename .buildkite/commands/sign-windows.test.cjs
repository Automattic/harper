const assert = require('node:assert/strict');
const { test } = require('node:test');
const { signWindows } = require('./sign-windows.cjs');

const env = {
	SIGNTOOL_PATH: 'C:\\Program Files\\Windows Kits\\signtool.exe',
	AZURE_CODE_SIGNING_DLIB: 'C:\\Signing Tools\\Azure.CodeSigning.Dlib.dll',
	AZURE_METADATA_JSON: 'C:\\Signing Tools\\metadata.json',
};

test('signs and verifies the same file using separate arguments, including spaces', () => {
	const file = 'C:\\Build Output\\Harper_2.12.0_x64-setup.exe';
	const calls = [];
	signWindows(file, env, (...args) => calls.push(args));
	assert.deepEqual(calls, [
		[
			env.SIGNTOOL_PATH,
			[
				'sign',
				'/v',
				'/fd',
				'SHA256',
				'/tr',
				'http://timestamp.acs.microsoft.com',
				'/td',
				'SHA256',
				'/dlib',
				env.AZURE_CODE_SIGNING_DLIB,
				'/dmdf',
				env.AZURE_METADATA_JSON,
				file,
			],
			{ stdio: 'inherit' },
		],
		[env.SIGNTOOL_PATH, ['verify', '/pa', '/all', '/v', '/tw', file], { stdio: 'inherit' }],
	]);
});

for (const name of Object.keys(env)) {
	for (const value of [undefined, '', '   ']) {
		test(`rejects missing or blank ${name}: ${JSON.stringify(value)}`, () => {
			assert.throws(
				() =>
					signWindows('Harper.exe', { ...env, [name]: value }, () => assert.fail('must not sign')),
				new RegExp(name),
			);
		});
	}
}

test('rejects an absent file before calling SignTool', () => {
	for (const file of [undefined, '', ' ']) {
		assert.throws(() => signWindows(file, env, () => assert.fail('must not sign')), /file to sign/);
	}
});

test('does not verify or report success after signing fails', () => {
	const failure = new Error('Azure signing failed');
	const commands = [];
	assert.throws(
		() =>
			signWindows('Harper.exe', env, (_, args) => {
				commands.push(args[0]);
				throw failure;
			}),
		(error) => error === failure,
	);
	assert.deepEqual(commands, ['sign']);
});

test('fails the hook if signature verification fails', () => {
	const failure = new Error('Signature verification failed');
	assert.throws(
		() =>
			signWindows('Harper.exe', env, (_, args) => {
				if (args[0] === 'verify') throw failure;
			}),
		(error) => error === failure,
	);
});

test('uses the toolkit timestamp server when supplied', () => {
	const calls = [];
	signWindows(
		'uninstall.exe',
		{ ...env, AZURE_TIMESTAMP_SERVER: 'http://timestamp.example.test' },
		(_, args) => calls.push(args),
	);
	assert.equal(calls[0][calls[0].indexOf('/tr') + 1], 'http://timestamp.example.test');
});
