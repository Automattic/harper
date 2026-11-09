const siteOrigin = 'https://writewithharper.com';

/** Page frontmatter fields used for document and social metadata. */
export interface MetadataInput {
	title?: string;
	description?: string;
}

/** Resolved document, canonical, and social metadata for a route. */
export interface ResolvedPageMetadata {
	title: string;
	description: string;
	canonical: string;
}

/** Use the production origin and a slash-normalized path, without query or fragment data. */
export function canonicalUrl(pathname: string): string {
	const url = new URL(siteOrigin);
	url.pathname = pathname.split(/[?#]/)[0].replace(/\/+$/, '') || '/';
	return url.href;
}

/** Use site metadata as fallback; marketing pages use their titles without a suffix. */
export function resolvePageMetadata(
	pathname: string,
	frontmatter: MetadataInput,
	site: Required<MetadataInput>,
	standaloneTitle = false,
): ResolvedPageMetadata {
	return {
		title: frontmatter.title
			? standaloneTitle
				? frontmatter.title
				: `${frontmatter.title} - ${site.title}`
			: site.title,
		description: frontmatter.description || site.description,
		canonical: canonicalUrl(pathname),
	};
}
