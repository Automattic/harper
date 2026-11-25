#!/usr/bin/env node
/**
 * Publishes the documentation in docs/ to the website.
 *
 *   node .github/scripts/publish-docs.mjs collect docs-files.json
 *   node .github/scripts/publish-docs.mjs publish docs-files.json --url https://example.com [--dry-run]
 *
 * The website does all the work of turning Markdown into pages. This script only gathers
 * files and sends the ones the site does not already have:
 *
 *   docs/**                     The Markdown pages and docs/sidebar.json, as they are.
 *   <any path>                  Files that a page shows with a code fence labelled file=<path>.
 *   generated/rules.json        Every rule's description and default, asked of harper.js.
 *   generated/reference/*.html  The harper.js API reference that docs.sh writes.
 *
 * `collect` needs harper.js built (`just build-harperjs`). `publish` needs nothing but Node,
 * so the job that holds a publishing token installs no dependencies.
 */

import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
	existsSync,
	readdirSync,
	readFileSync,
	realpathSync,
	statSync,
	writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const REFERENCE_DIR = 'packages/web/static/docs/harperjs/ref';
const MAX_FILE_BYTES = 1024 * 1024;
const MAX_BATCH_BYTES = 400_000;
const MAX_BATCH_FILES = 40;

/**
 * Gather every file the site needs, by the path the site knows it under.
 *
 * @param {string} repo The repository root.
 * @param {object} [options]
 * @param {() => Promise<object>} [options.rules] Returns the rule descriptions and defaults.
 * @param {string} [options.referenceDir] Where docs.sh wrote the API reference.
 * @returns {Promise<Record<string, string>>}
 */
export async function collect(repo, options = {}) {
	const root = realpathSync(repo);
	const { rules = () => loadRules(root), referenceDir = path.join(root, REFERENCE_DIR) } = options;
	const files = {};

	for (const file of walk(path.join(root, 'docs'))) {
		if (file.endsWith('.md') || file.endsWith('.json')) {
			files[path.posix.join('docs', file)] = readFileSync(path.join(root, 'docs', file), 'utf8');
		}
	}

	// A page shows a file from the repository with an empty code fence labelled file=<path>.
	for (const [name, content] of Object.entries(files)) {
		if (!name.endsWith('.md')) {
			continue;
		}

		for (const [, shown] of content.matchAll(/^\s*(?:```+|~~~+)[^\n`]*?\bfile=(\S+)/gm)) {
			const target = path.resolve(root, shown);

			if (isPublishable(root, target)) {
				files[shown] = readFileSync(target, 'utf8');
			}
		}
	}

	files['generated/rules.json'] = JSON.stringify(await rules());

	if (!existsSync(referenceDir)) {
		throw new Error(`${referenceDir} does not exist. Run "just build-harperjs" first.`);
	}

	for (const file of readdirSync(referenceDir).sort()) {
		if (file.endsWith('.html')) {
			files[`generated/reference/${file}`] = readFileSync(path.join(referenceDir, file), 'utf8');
		}
	}

	return files;
}

/** Only ordinary files inside the checkout, and nothing from .git. */
function isPublishable(root, target) {
	if (!existsSync(target) || !statSync(target).isFile() || statSync(target).size > MAX_FILE_BYTES) {
		return false;
	}

	const inside = path.relative(root, realpathSync(target));

	return (
		!inside.startsWith('..') && !path.isAbsolute(inside) && inside.split(path.sep)[0] !== '.git'
	);
}

function* walk(directory, prefix = '') {
	for (const entry of readdirSync(path.join(directory, prefix), { withFileTypes: true })) {
		const relative = path.posix.join(prefix, entry.name);

		if (entry.isDirectory()) {
			yield* walk(directory, relative);
		} else if (entry.isFile()) {
			yield relative;
		}
	}
}

/** Ask harper.js for every rule's description and whether it is on by default. */
async function loadRules(root) {
	const dist = path.join(root, 'packages/harper.js/dist');

	if (!existsSync(path.join(dist, 'index.js'))) {
		throw new Error('harper.js is not built. Run "just build-harperjs" first.');
	}

	const { LocalLinter } = await import(pathToFileURL(path.join(dist, 'index.js')));
	const { slimBinaryInlined } = await import(
		pathToFileURL(path.join(dist, 'slimBinaryInlined.js'))
	);
	const linter = new LocalLinter({ binary: slimBinaryInlined });

	try {
		const [descriptions, defaultConfig] = await Promise.all([
			linter.getLintDescriptionsHTML(),
			linter.getDefaultLintConfig(),
		]);

		return { descriptions, defaultConfig };
	} finally {
		await linter.dispose();
	}
}

/** The audience the site expects in a token, tied to its host name. */
export function audienceFor(siteUrl) {
	return `harper-docs-sync:${new URL(siteUrl).hostname.toLowerCase()}`;
}

/**
 * Ask GitHub for a token that says which repository, branch and workflow this run belongs to.
 * The site checks that instead of a shared secret, so no credential is stored anywhere.
 */
export async function githubToken(audience, env = process.env, fetchImpl = fetch) {
	if (env.HARPER_DOCS_SYNC_TOKEN) {
		// A locally signed token, for trying a publish against a development site.
		return env.HARPER_DOCS_SYNC_TOKEN;
	}

	const { ACTIONS_ID_TOKEN_REQUEST_URL: url, ACTIONS_ID_TOKEN_REQUEST_TOKEN: requestToken } = env;

	if (!url || !requestToken) {
		throw new Error(
			'No token available. In GitHub Actions the job needs "permissions: id-token: write".',
		);
	}

	const response = await fetchImpl(`${url}&audience=${encodeURIComponent(audience)}`, {
		headers: { authorization: `bearer ${requestToken}` },
	});

	if (!response.ok) {
		throw new Error(`GitHub did not issue a token (HTTP ${response.status}).`);
	}

	return (await response.json()).value;
}

/**
 * Send the files to the site: a list of every file with a hash, then only the files the site
 * lacks, then a last call that has the site rebuild its pages and remove the ones that are
 * gone. Publishing twice in a row sends nothing the second time.
 *
 * @param {Record<string, string>} files From collect().
 * @param {object} options
 * @param {string} options.url The site, such as https://writewithharper.com.
 * @param {string} [options.commit] The commit the files came from.
 * @param {boolean} [options.dryRun] Report what would be sent and stop.
 * @param {(audience: string) => Promise<string>} [options.token]
 * @param {typeof fetch} [options.fetch]
 * @param {(line: string) => void} [options.log]
 */
export async function publish(files, options) {
	const {
		url,
		commit = '',
		dryRun = false,
		token = githubToken,
		fetch: fetchImpl = fetch,
		log = console.log,
	} = options;

	const audience = audienceFor(url);
	const manifest = {
		commit,
		files: Object.fromEntries(
			Object.entries(files).map(([name, content]) => [
				name,
				createHash('sha256').update(content).digest('hex'),
			]),
		),
	};

	async function call(step, body) {
		// rest_route works whether or not the site has pretty permalinks.
		const response = await fetchImpl(new URL(`/?rest_route=/harper-docs/v1/${step}`, url), {
			method: 'POST',
			headers: {
				'content-type': 'application/json',
				authorization: `Bearer ${await token(audience)}`,
			},
			body: JSON.stringify(body),
		});

		const text = await response.text();
		let data;

		try {
			data = JSON.parse(text);
		} catch {
			throw new Error(`${step}: the site answered HTTP ${response.status} without JSON.`);
		}

		if (!response.ok) {
			throw new Error(`${step}: ${data.message ?? `HTTP ${response.status}`}`);
		}

		return data;
	}

	const plan = await call('plan', manifest);

	log(
		`${plan.needed.length} to send, ${plan.unchanged} unchanged, ${plan.remove.length} to remove.`,
	);

	for (const name of plan.needed) {
		log(`  send    ${name}`);
	}
	for (const name of plan.remove) {
		log(`  remove  ${name}`);
	}

	if (dryRun) {
		log('Dry run: nothing was changed.');
		return plan;
	}

	for (const batch of batches(plan.needed.map((name) => ({ path: name, content: files[name] })))) {
		await call('files', { commit, files: batch });
	}

	const finished = await call('finish', manifest);

	for (const warning of finished.warnings) {
		// Shown as a warning on the workflow run.
		log(`::warning::${warning}`);
	}

	log(`Done: ${finished.updated.length} page(s) updated, ${finished.trashed.length} removed.`);

	return { ...plan, ...finished };
}

/** Split files so no request is large enough to hit a body limit. */
function* batches(entries) {
	let batch = [];
	let bytes = 0;

	for (const entry of entries) {
		const size = Buffer.byteLength(entry.content);

		if (batch.length > 0 && (bytes + size > MAX_BATCH_BYTES || batch.length >= MAX_BATCH_FILES)) {
			yield batch;
			batch = [];
			bytes = 0;
		}

		batch.push(entry);
		bytes += size;
	}

	if (batch.length > 0) {
		yield batch;
	}
}

async function main(argv) {
	const [command, file, ...flags] = argv;
	const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

	if (command === 'collect' && file) {
		const files = await collect(repo);
		let commit = '';

		try {
			commit = execFileSync('git', ['-C', repo, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
		} catch {
			// Not a checkout. The site only records the commit, so publish without it.
		}

		writeFileSync(file, JSON.stringify({ commit, files }));
		console.log(`Collected ${Object.keys(files).length} files into ${file}`);
		return;
	}

	const url = flags[flags.indexOf('--url') + 1];

	if (command === 'publish' && file && flags.includes('--url') && url) {
		const { commit, files } = JSON.parse(readFileSync(file, 'utf8'));
		await publish(files, { url, commit, dryRun: flags.includes('--dry-run') });
		return;
	}

	console.error(
		'Usage: publish-docs.mjs collect <file>\n       publish-docs.mjs publish <file> --url <site> [--dry-run]',
	);
	process.exitCode = 2;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
	main(process.argv.slice(2)).catch((error) => {
		console.error(`::error::${error.message}`);
		process.exitCode = 1;
	});
}
