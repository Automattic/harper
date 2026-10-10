const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const {
	copyFileSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	rmSync,
	writeFileSync,
} = require('node:fs');
const { tmpdir } = require('node:os');
const { join } = require('node:path');
const { test } = require('node:test');

for (const scenario of ['success', 'wrong-tag', 'download-failure', 'missing-artifact']) {
	test(`Windows release upload: ${scenario}`, () => {
		const root = mkdtempSync(join(tmpdir(), 'Harper release '));
		try {
			const commands = join(root, '.buildkite/commands');
			const bin = join(root, 'bin');
			const trace = join(root, 'calls.jsonl');
			mkdirSync(commands, { recursive: true });
			mkdirSync(bin);
			mkdirSync(join(root, 'harper-desktop/src-tauri'), { recursive: true });
			writeFileSync(join(root, 'harper-desktop/src-tauri/tauri.conf.json'), '{"version":"9.0.0"}');
			copyFileSync(join(__dirname, 'release-desktop-windows.sh'), join(commands, 'release.sh'));
			for (const name of ['buildkite-agent', 'install_gems', 'bundle']) {
				writeFileSync(
					join(bin, name),
					`#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const args = process.argv.slice(2);
const name = path.basename(process.argv[1]);
fs.appendFileSync(process.env.HARPER_TEST_TRACE, JSON.stringify([name, ...args]) + '\\n');
if (name === 'buildkite-agent') {
  if (process.env.HARPER_TEST_SCENARIO === 'download-failure') process.exit(1);
  if (process.env.HARPER_TEST_SCENARIO !== 'missing-artifact') {
    fs.mkdirSync(path.dirname(args[2]), { recursive: true });
    fs.writeFileSync(args[2], 'mock installer');
  }
}
`,
					{ mode: 0o755 },
				);
			}
			const result = spawnSync('bash', [join(commands, 'release.sh')], {
				encoding: 'utf8',
				env: {
					...process.env,
					PATH: `${bin}:${process.env.PATH}`,
					BUILDKITE_TAG: scenario === 'wrong-tag' ? 'v1.0.0' : 'v9.0.0',
					HARPER_TEST_SCENARIO: scenario,
					HARPER_TEST_TRACE: trace,
				},
			});
			if (scenario === 'wrong-tag') {
				assert.notEqual(result.status, 0);
				assert.match(result.stderr, /Release tag must match/);
				return;
			}
			const calls = readFileSync(trace, 'utf8').trim().split('\n').map(JSON.parse);
			const installer =
				'harper-desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/Harper_9.0.0_x64-setup.exe';
			assert.deepEqual(calls[0], [
				'buildkite-agent',
				'artifact',
				'download',
				installer,
				'.',
				'--step',
				'build-desktop-windows',
			]);
			if (scenario === 'success') {
				assert.equal(result.status, 0, result.stderr);
				assert.deepEqual(calls[2], [
					'bundle',
					'exec',
					'fastlane',
					'upload_windows_github_release',
					'tag:v9.0.0',
					`installer:${installer}`,
				]);
			} else {
				assert.notEqual(result.status, 0);
				assert.equal(calls.length, 1, 'must not install gems or upload after a failed download');
			}
		} finally {
			rmSync(root, { recursive: true, force: true });
		}
	});
}
