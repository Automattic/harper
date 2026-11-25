/**
 * One docs page: plain Markdown in, WordPress blocks out.
 *
 * Pages may not contain raw HTML. Three things Markdown cannot say are written as fenced code
 * blocks with a special label, which still read as ordinary code blocks on GitHub:
 *
 *   ```ts file=packages/harper.js/src/Linter.ts     an empty fence that shows a file from the repo
 *   ```harper-editor                                sample text for the live editor
 *   ```harper-rules                                 the generated table of rules
 */

import { toHtml } from 'hast-util-to-html';
import lisp from 'highlight.js/lib/languages/lisp';
import { common, createLowlight } from 'lowlight';
import { toHast } from 'mdast-util-to-hast';
import { toString as mdastToString } from 'mdast-util-to-string';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import { unified } from 'unified';
import { SKIP, visit } from 'unist-util-visit';
import { parse as parseYaml } from 'yaml';
import { block, escapeHtml } from './blocks.js';
import { headingId, uniqueId } from './slug.js';

const lowlight = createLowlight(common);
lowlight.register({ lisp });
lowlight.registerAlias({ lisp: ['elisp'] });

const parser = unified().use(remarkParse).use(remarkFrontmatter, ['yaml']).use(remarkGfm);

const HTML_OPTIONS = { closeSelfClosing: true, characterReferences: { useNamedReferences: true } };
const DIRECTIVE = /^@@harper-docs-directive-(\d+)@@$/;

/** A problem in a docs file, reported with where it is. */
export class DocsProblem extends Error {
	constructor(message, file, node) {
		const line = node?.position?.start?.line;
		super(`${file}${line ? `:${line}` : ''}: ${message}`);
	}
}

/**
 * Parse a page, read its frontmatter and give every heading below the title an id.
 *
 * @returns {{ title: string, description: string, tree: object, headings: {id: string, text: string, depth: number}[] }}
 */
export function parsePage(source, file) {
	const tree = parser.parse(source);
	const frontmatter = tree.children[0]?.type === 'yaml' ? parseYaml(tree.children[0].value) : null;

	if (typeof frontmatter?.title !== 'string' || typeof frontmatter?.description !== 'string') {
		throw new DocsProblem('needs frontmatter with a title and a description', file);
	}

	tree.children = tree.children.filter((node) => node.type !== 'yaml');

	visit(tree, 'html', (node) => {
		throw new DocsProblem(
			`raw HTML is not allowed in docs (${node.value.slice(0, 40).trim()}). Put literal angle brackets in backticks.`,
			file,
			node,
		);
	});

	// The page title is printed by the site, so the matching first heading goes.
	if (tree.children[0]?.type === 'heading' && tree.children[0].depth === 1) {
		tree.children.shift();
	}

	const used = new Set();
	const headings = [];

	visit(tree, 'heading', (node) => {
		if (node.depth === 1) {
			throw new DocsProblem('only the first heading may be a top-level "#" heading', file, node);
		}

		const text = mdastToString(node);
		const id = uniqueId(headingId(text), used);

		node.data = { hProperties: { className: ['wp-block-heading'], id } };
		headings.push({ id, text, depth: node.depth });
	});

	return { title: frontmatter.title, description: frontmatter.description, tree, headings };
}

/**
 * Turn a parsed page into block markup.
 *
 * @param {object} tree From parsePage.
 * @param {object} context
 * @param {string} context.file For error messages.
 * @param {(url: string, node: object) => string} context.resolveLink Rewrites a link target.
 * @param {(url: string, node: object) => string} context.resolveImage Rewrites an image source.
 * @param {(path: string, node: object) => string} context.include Reads a file from the repo.
 * @param {() => string} context.rulesTable Block markup for the table of rules.
 */
export function renderPage(tree, context) {
	const directives = [];

	visit(tree, 'link', (node) => {
		node.url = context.resolveLink(node.url, node);
	});
	visit(tree, 'image', (node) => {
		node.url = context.resolveImage(node.url, node);
	});

	visit(tree, 'code', (node, index, parent) => {
		const directive = renderDirective(node, context);

		if (directive !== null) {
			if (parent !== tree) {
				throw new DocsProblem(
					`a "${node.lang}" block cannot sit inside a list or a quote`,
					context.file,
					node,
				);
			}

			directives.push(directive);
			parent.children[index] = {
				type: 'paragraph',
				children: [{ type: 'text', value: `@@harper-docs-directive-${directives.length - 1}@@` }],
			};
			return SKIP;
		}

		highlight(node, context);
	});

	const blocks = [];

	for (const element of toHast(tree).children) {
		if (element.type !== 'element') {
			continue;
		}

		const marker = element.tagName === 'p' && textOf(element).match(DIRECTIVE);
		blocks.push(marker ? directives[Number(marker[1])] : elementToBlock(element));
	}

	return blocks.join('\n\n');
}

/** The special fences. Returns block markup, or null for an ordinary code block. */
function renderDirective(node, context) {
	if (node.lang === 'harper-rules') {
		return context.rulesTable();
	}

	if (node.lang === 'harper-editor') {
		// Plain text until the site's editor bundle finds this element and takes it over.
		const paragraphs = node.value
			.split(/\n\s*\n/)
			.map((text) => `<p>${escapeHtml(text.trim()).replace(/\n/g, '<br />')}</p>`)
			.join('');

		return block(
			'html',
			null,
			`<div data-docs-embed="editor" data-harper-editor>${paragraphs}</div>`,
		);
	}

	return null;
}

/** Apply syntax highlighting now, so the site ships no highlighter. */
function highlight(node, context) {
	const file = node.meta?.match(/(?:^|\s)file=(\S+)/)?.[1];

	if (file) {
		if (node.value.trim() !== '') {
			throw new DocsProblem(
				`a code block with file=${file} must be empty; its contents come from that file`,
				context.file,
				node,
			);
		}

		node.value = context.include(file, node).replace(/\n+$/, '');
	}

	if (!node.lang) {
		return;
	}

	const className = ['hljs', `language-${node.lang}`];

	node.data = lowlight.registered(node.lang)
		? { hProperties: { className }, hChildren: lowlight.highlight(node.lang, node.value).children }
		: { hProperties: { className } };
}

function elementToBlock(element) {
	switch (element.tagName) {
		case 'h2':
		case 'h3':
		case 'h4':
		case 'h5':
		case 'h6': {
			const level = Number(element.tagName[1]);
			return block('heading', level === 2 ? null : { level }, html(element));
		}

		case 'p': {
			const children = meaningful(element.children);

			if (children.length === 1 && children[0].tagName === 'img') {
				return block('image', null, `<figure class="wp-block-image">${html(children[0])}</figure>`);
			}

			return block('paragraph', null, html(element));
		}

		case 'ul':
		case 'ol':
			return isSimpleList(element) ? listBlock(element) : block('html', null, html(element));

		case 'blockquote': {
			const children = meaningful(element.children);

			if (children.every((child) => child.tagName === 'p')) {
				const paragraphs = children.map((child) => block('paragraph', null, html(child)));
				return block(
					'quote',
					null,
					`<blockquote class="wp-block-quote">${paragraphs.join('\n')}</blockquote>`,
				);
			}

			return block('html', null, html(element));
		}

		case 'table':
			return block(
				'table',
				{ hasFixedLayout: false },
				`<figure class="wp-block-table">${html(element)}</figure>`,
			);

		case 'hr':
			return block(
				'separator',
				null,
				'<hr class="wp-block-separator has-alpha-channel-opacity" />',
			);

		default:
			// Code blocks, and anything without a matching core block.
			return block('html', null, html(element));
	}
}

/** A list the core List block can hold: each item is text, optionally followed by one such list. */
function isSimpleList(list) {
	return meaningful(list.children).every((item) => {
		const { nested, content } = splitItem(item);
		const blockLevel = content.filter(
			(child) =>
				child.type === 'element' && ['p', 'pre', 'blockquote', 'table'].includes(child.tagName),
		);

		return (
			item.properties?.className === undefined &&
			(blockLevel.length === 0 ||
				(blockLevel.length === 1 && content.length === 1 && blockLevel[0].tagName === 'p')) &&
			nested.length <= 1 &&
			nested.every(isSimpleList)
		);
	});
}

function listBlock(list) {
	const ordered = list.tagName === 'ol';
	const start = ordered && list.properties?.start > 1 ? list.properties.start : null;
	const attributes = ordered ? { ordered: true, ...(start ? { start } : {}) } : null;

	const items = meaningful(list.children).map((item) => {
		const { nested, content } = splitItem(item);
		const inline =
			content.length === 1 && content[0].tagName === 'p' ? content[0].children : content;
		const text = toHtml({ type: 'root', children: inline }, HTML_OPTIONS).trim();

		return block('list-item', null, `<li>${text}${nested.map(listBlock).join('')}</li>`);
	});

	const open = ordered
		? `<ol${start ? ` start="${start}"` : ''} class="wp-block-list">`
		: '<ul class="wp-block-list">';

	return block('list', attributes, `${open}${items.join('\n')}</${list.tagName}>`);
}

function splitItem(item) {
	const children = meaningful(item.children);
	const isList = (child) => child.type === 'element' && ['ul', 'ol'].includes(child.tagName);

	return { nested: children.filter(isList), content: children.filter((child) => !isList(child)) };
}

/** Children that are not whitespace between tags. */
function meaningful(children) {
	return children.filter((child) => child.type !== 'text' || child.value.trim() !== '');
}

function html(element) {
	return toHtml(element, HTML_OPTIONS);
}

function textOf(node) {
	return node.type === 'text' ? node.value : (node.children ?? []).map(textOf).join('');
}
