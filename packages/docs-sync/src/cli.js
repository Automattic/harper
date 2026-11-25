#!/usr/bin/env node
/**
 * node src/cli.js [--out dist/bundle.json] [--no-reference]
 *
 * Builds the docs bundle and fails, listing every problem, if the docs are not publishable.
 * Sending the bundle to the site is src/publish.js.
 */

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { buildBundle, DocsBuildError } from './build.js';
import { loadRules } from './rules.js';

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repo = path.resolve(packageRoot, '../..');

const { values } = parseArgs({
	options: {
		out: { type: 'string', default: 'dist/bundle.json' },
		reference: { type: 'string', default: 'packages/web/static/docs/harperjs/ref' },
		'no-reference': { type: 'boolean', default: false },
	},
});

async function build() {
	const referenceDir = values['no-reference'] ? null : path.resolve(repo, values.reference);

	if (referenceDir && !existsSync(referenceDir)) {
		throw new Error(
			`${values.reference} does not exist. Run "just build-harperjs" first, or pass --no-reference.`,
		);
	}

	const bundle = buildBundle({
		repo,
		referenceDir,
		rules: await loadRules(),
		clientLinks: JSON.parse(readFileSync(path.join(packageRoot, 'client-links.json'), 'utf8')),
		commit: git('rev-parse', 'HEAD'),
	});

	const out = path.resolve(packageRoot, values.out);
	mkdirSync(path.dirname(out), { recursive: true });
	writeFileSync(out, JSON.stringify(bundle));

	console.log(
		`Built ${bundle.pages.length} pages and ${bundle.reference.length} reference files into ${path.relative(process.cwd(), out)}`,
	);
}

function git(...args) {
	try {
		return execFileSync('git', ['-C', repo, ...args], { encoding: 'utf8' }).trim();
	} catch {
		return '';
	}
}

try {
	await build();
} catch (error) {
	console.error(error instanceof DocsBuildError ? error.message : error);
	process.exit(1);
}
