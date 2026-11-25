/**
 * WordPress block markup. A block is an HTML comment around ordinary HTML, which is what lets
 * the site's theme style each piece of a docs page the way it styles any other content.
 */

/** Serialize attributes as WordPress does, so no value can end the comment early. */
export function blockAttributes(attributes) {
	return JSON.stringify(attributes)
		.replace(/--/g, '\\u002d\\u002d')
		.replace(/</g, '\\u003c')
		.replace(/>/g, '\\u003e')
		.replace(/&/g, '\\u0026')
		.replace(/\\"/g, '\\u0022');
}

export function block(name, attributes, html) {
	const serialized =
		attributes && Object.keys(attributes).length > 0 ? ` ${blockAttributes(attributes)}` : '';

	return `<!-- wp:${name}${serialized} -->\n${html}\n<!-- /wp:${name} -->`;
}

export function escapeHtml(text) {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}
