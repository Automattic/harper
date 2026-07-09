#! /bin/node

import { spawnSync } from 'child_process';
import { chromium } from 'playwright';

const chromiumExecutablePath = chromium.executablePath();
const args = process.argv.slice(2).join(' ');

spawnSync(`vglrun -d :0 -- ${chromiumExecutablePath} ${args}`, {
	stdio: [0, 1, 2, 3, 4],
	shell: true,
});
