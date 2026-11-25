#!/usr/bin/env node
/**
 * node src/publish.js --url https://example.com [--bundle dist/bundle.json] [--dry-run]
 *
 * Sends a bundle that `node src/cli.js` made. Kept apart from the build so that it
 * imports nothing but this package's own sync.js: the publishing job runs it without
 * installing any dependencies.
 */

import { readFileSync } from 'node:fs';
import { parseArgs } from 'node:util';
import { sync } from './sync.js';

const { values } = parseArgs({
	options: {
		url: { type: 'string' },
		bundle: { type: 'string', default: 'dist/bundle.json' },
		'dry-run': { type: 'boolean', default: false },
	},
});

if (!values.url) {
	console.error('Usage: node src/publish.js --url https://example.com [--dry-run]');
	process.exit(2);
}

try {
	await sync(JSON.parse(readFileSync(values.bundle, 'utf8')), {
		url: values.url,
		dryRun: values['dry-run'],
	});
} catch (error) {
	console.error(`::error::${error.message}`);
	process.exit(1);
}
