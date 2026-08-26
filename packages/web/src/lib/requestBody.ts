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
	const isJson = request.headers.get('content-type')?.startsWith('application/json');

	if (isJson) {
		const json = await request.json();
		return { get: (name: string) => (name in json ? (json[name] ?? null) : null) };
	}

	const form = await request.formData();
	// These routes never accept file fields; treat one as absent rather than
	// forwarding a File where a string is expected.
	return {
		get: (name: string) => (typeof form.get(name) === 'string' ? (form.get(name) as string) : null),
	};
}
