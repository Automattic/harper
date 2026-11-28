#!/usr/bin/env node

import { spawnSync } from 'child_process';
import { chromium } from 'playwright';

const chromiumExecutablePath = chromium.executablePath();
const args = process.argv.slice(2);

function hasProgram(program) {
	const result = spawnSync(program, ['--version'], { stdio: 'ignore' });
	return result.error?.code !== 'ENOENT';
}

// Pass arguments directly rather than through a shell: on macOS the Chromium path contains spaces.
const stdio = [0, 1, 2, 3, 4];
let result;

if (hasProgram('vglrun')) {
	console.log('Has vglrun');
	result = spawnSync(
		'vglrun',
		['-d', process.env.DISPLAY ?? '', '--', chromiumExecutablePath, ...args],
		{ stdio },
	);
} else {
	console.log('Does not have vglrun');
	result = spawnSync(chromiumExecutablePath, args, { stdio });
}

process.exitCode = result.status ?? 1;
