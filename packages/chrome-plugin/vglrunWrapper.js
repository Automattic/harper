#! /bin/node

import { spawnSync } from 'child_process';
import { chromium } from 'playwright';

const chromiumExecutablePath = chromium.executablePath();
const args = process.argv.slice(2).join(' ');

function hasProgram(program) {
  const result = spawnSync(program, ['--version'], { stdio: 'ignore' });
  return result.error?.code !== 'ENOENT';
}

if (hasProgram("vglrun")){
  spawnSync(`vglrun -d :0 -- ${chromiumExecutablePath} ${args}`, {
  	stdio: [0, 1, 2, 3, 4],
  	shell: true,
  });
}else{
   spawnSync(`${chromiumExecutablePath} ${args}`, {
  	stdio: [0, 1, 2, 3, 4],
  	shell: true,
  });
}
