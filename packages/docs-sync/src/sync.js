/**
 * Send a bundle to the site.
 *
 * The site keeps a hash for everything it has stored, so a sync is three steps: ask which
 * entries the site lacks, upload those, then let it remove what the docs no longer contain.
 * Nothing here needs a dependency, which keeps third-party code out of the one job that holds
 * a token.
 */

const MAX_BATCH_BYTES = 400_000;
const MAX_BATCH_ENTRIES = 40;

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
		// A locally signed token, for trying the sync against a development site.
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
 * @param {object} bundle From buildBundle().
 * @param {object} options
 * @param {string} options.url The site, such as https://writewithharper.com.
 * @param {boolean} [options.dryRun] Report what would change and stop.
 * @param {(audience: string) => Promise<string>} [options.token]
 * @param {typeof fetch} [options.fetch]
 * @param {(line: string) => void} [options.log]
 */
export async function sync(bundle, options) {
	const {
		url,
		dryRun = false,
		token = githubToken,
		fetch: fetchImpl = fetch,
		log = console.log,
	} = options;

	const audience = audienceFor(url);
	const manifest = {
		source: bundle.source,
		sidebar_hash: bundle.sidebar_hash,
		pages: bundle.pages.map(({ path, hash }) => ({ path, hash })),
		reference: bundle.reference.map(({ file, hash }) => ({ file, hash })),
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
			const failed = data.failed ? ` ${JSON.stringify(data.failed)}` : '';
			throw new Error(`${step}: ${data.message ?? `HTTP ${response.status}`}${failed}`);
		}

		return data;
	}

	const plan = await call('plan', manifest);
	const remove = [...plan.pages.remove, ...plan.reference.remove];

	log(
		`Pages: ${plan.pages.needed.length} to send, ${plan.pages.unchanged} unchanged, ${plan.pages.remove.length} to remove.`,
	);
	log(
		`Reference files: ${plan.reference.needed.length} to send, ${plan.reference.unchanged} unchanged, ${plan.reference.remove.length} to remove.`,
	);
	log(`Sidebar: ${plan.sidebar_changed ? 'changed' : 'unchanged'}.`);

	for (const name of [...plan.pages.needed, ...plan.reference.needed]) {
		log(`  send    ${name}`);
	}
	for (const name of remove) {
		log(`  remove  ${name}`);
	}

	if (dryRun) {
		log('Dry run: nothing was changed.');
		return { ...plan, sent: 0 };
	}

	// Parents are listed before their children in the bundle, and are sent in that order.
	const pages = bundle.pages.filter((entry) => plan.pages.needed.includes(entry.path));
	const reference = bundle.reference.filter((entry) => plan.reference.needed.includes(entry.file));
	let sent = 0;

	for (const [key, entries] of [
		['pages', pages],
		['reference', reference],
	]) {
		for (const batch of batches(entries)) {
			const result = await call('items', { source: bundle.source, [key]: batch });

			for (const name of result.filtered) {
				// Shown as a warning on the workflow run.
				log(`::warning::The site removed markup it does not allow from ${name}.`);
			}

			sent += result.stored.length;
		}
	}

	const finished = await call('finish', { ...manifest, sidebar: bundle.sidebar });

	log(
		`Done: sent ${sent}, removed ${finished.trashed.length + finished.deleted.length}${finished.sidebar_changed ? ', sidebar updated' : ''}.`,
	);

	return { ...plan, sent };
}

/** Split entries so no request is large enough to hit a body limit. */
function* batches(entries) {
	let batch = [];
	let bytes = 0;

	for (const entry of entries) {
		const size = Buffer.byteLength(entry.content);

		if (batch.length > 0 && (bytes + size > MAX_BATCH_BYTES || batch.length >= MAX_BATCH_ENTRIES)) {
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
