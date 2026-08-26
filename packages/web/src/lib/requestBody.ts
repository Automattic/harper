import { error } from '@sveltejs/kit';

/**
 * Parses a POST body that may arrive as either JSON or a native form submission
 * (`application/x-www-form-urlencoded` or `multipart/form-data`), exposing both
 * through the same `FormData`-style `get()` accessor.
 *
 * The browser extension sends JSON specifically so that SvelteKit's CSRF
 * `checkOrigin` (which only gates `application/x-www-form-urlencoded`,
 * `multipart/form-data`, and `text/plain`) doesn't reject its cross-origin
 * request; the website's own `<form>` elements submit url-encoded, same-origin,
 * and are unaffected by that check either way.
 */
export async function parseRequestBody(
	request: Request,
): Promise<{ get(name: string): string | null }> {
	const contentType =
		request.headers.get('content-type')?.split(';', 1)[0].trim().toLowerCase() ?? '';
	const isJson = contentType === 'application/json';

	if (isJson) {
		let json: unknown;
		try {
			json = await request.json();
		} catch {
			throw error(400, 'Malformed JSON body.');
		}

		if (typeof json !== 'object' || json === null || Array.isArray(json)) {
			throw error(400, 'JSON body must be an object.');
		}

		const record = json as Record<string, unknown>;
		return {
			get: (name: string) => (typeof record[name] === 'string' ? record[name] : null),
		};
	}

	const form = await request.formData();
	// These routes never accept file fields; treat one as absent rather than
	// forwarding a File where a string is expected.
	return {
		get: (name: string) => (typeof form.get(name) === 'string' ? (form.get(name) as string) : null),
	};
}
