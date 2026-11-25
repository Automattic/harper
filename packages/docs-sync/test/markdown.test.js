import assert from 'node:assert/strict';
import { test } from 'node:test';
import { blockAttributes } from '../src/blocks.js';
import { DocsBuildError } from '../src/build.js';
import { headingId } from '../src/slug.js';
import { build, page, render } from './helpers.js';

const problems = (files, options) => {
	try {
		build(files, options);
	} catch (error) {
		assert.ok(error instanceof DocsBuildError, String(error));
		return error.problems;
	}
	return [];
};

test('heading ids keep capitals and drop punctuation', () => {
	assert.equal(headingId('Updating the Extension'), 'Updating-the-Extension');
	assert.equal(headingId("What's the Difference?"), 'Whats-the-Difference');
	assert.equal(headingId('Prerelease Checks + Testing'), 'Prerelease-Checks-Testing');
	assert.equal(headingId('harper-core'), 'harper-core');
});

test('the title heading is dropped and other headings get ids', () => {
	const content = render('## Code Actions\n\n### `harper-ls` Setup\n\n## Code Actions');

	assert.doesNotMatch(content, /<h1/);
	assert.match(
		content,
		/<!-- wp:heading -->\n<h2 class="wp-block-heading" id="Code-Actions">Code Actions<\/h2>/,
	);
	assert.match(
		content,
		/<!-- wp:heading {"level":3} -->\n<h3 class="wp-block-heading" id="harper-ls-Setup"><code>harper-ls<\/code> Setup<\/h3>/,
	);
	assert.match(content, /id="Code-Actions-1"/);
});

test('prose becomes core blocks', () => {
	const content = render(
		[
			'A *short* paragraph with `code`.',
			'- one\n- two\n    - nested',
			'3. third\n4. fourth',
			'> Quoted.',
			'| A | B |\n| --- | --- |\n| 1 | 2 |',
			'![Alt text](/images/shot.png)',
			'---',
		].join('\n\n'),
		{ 'static/images/shot.png': 'png' },
	);

	assert.match(
		content,
		/<!-- wp:paragraph -->\n<p>A <em>short<\/em> paragraph with <code>code<\/code>.<\/p>/,
	);
	assert.match(content, /<ul class="wp-block-list"><!-- wp:list-item -->\n<li>one<\/li>/);
	assert.match(
		content,
		/<li>two<!-- wp:list -->\n<ul class="wp-block-list"><!-- wp:list-item -->\n<li>nested<\/li>/,
	);
	assert.match(
		content,
		/<!-- wp:list {"ordered":true,"start":3} -->\n<ol start="3" class="wp-block-list">/,
	);
	assert.match(
		content,
		/<blockquote class="wp-block-quote"><!-- wp:paragraph -->\n<p>Quoted.<\/p>/,
	);
	assert.match(
		content,
		/<!-- wp:table {"hasFixedLayout":false} -->\n<figure class="wp-block-table"><table>/,
	);
	assert.match(
		content,
		/<!-- wp:image -->\n<figure class="wp-block-image"><img src="\/images\/shot.png" alt="Alt text" \/><\/figure>/,
	);
	assert.match(content, /<!-- wp:separator -->/);
});

test('a list item that holds a code block stays plain HTML', () => {
	const content = render('1. Run this:\n\n    ```bash\n    just setup\n    ```\n\n2. Done.');

	assert.match(content, /^<!-- wp:html -->\n<ol>/);
	assert.match(content, /<pre><code class="hljs language-bash">/);
});

test('code is highlighted at build time and escaped', () => {
	const content = render(
		'```rust\nlet a = "<b>";\n```\n\n```\nplain <text>\n```\n\n```nosuchlanguage\nx\n```',
	);

	assert.match(
		content,
		/<pre><code class="hljs language-rust"><span class="hljs-keyword">let<\/span>/,
	);
	assert.match(content, /"&lt;b>"/);
	assert.match(content, /<pre><code>plain (&#x3C;|&lt;)text/);
	assert.match(content, /<code class="hljs language-nosuchlanguage">x/);
});

test('raw HTML is refused', () => {
	assert.match(
		problems({ 'docs/about.md': page('About', '<script>alert(1)</script>') })[0],
		/docs\/about\.md:8: raw HTML is not allowed/,
	);
	assert.match(problems({ 'docs/about.md': page('About', 'Inline <b>bold</b>.') })[0], /raw HTML/);
});

test('frontmatter is required', () => {
	assert.match(problems({ 'docs/about.md': '# About\n' })[0], /needs frontmatter/);
});

test('an empty fence with file= shows that file', () => {
	const content = render('```ts file=src/example.ts\n```', {
		'src/example.ts': 'export const answer: number = 42;\n',
	});

	assert.match(content, /<code class="hljs language-ts"><span class="hljs-keyword">export<\/span>/);
	assert.match(content, /42/);
});

test('file= cannot leave the repository or be filled in by hand', () => {
	assert.match(
		problems({ 'docs/about.md': page('About', '```txt file=../outside.txt\n```') })[0],
		/does not exist|outside the repository/,
	);
	assert.match(
		problems({
			'docs/about.md': page('About', '```txt file=.git/config\n```'),
			'.git/config': 'x',
		})[0],
		/outside the repository/,
	);
	assert.match(
		problems({
			'docs/about.md': page('About', '```js file=a.js\nconsole.log(1)\n```'),
			'a.js': '',
		})[0],
		/must be empty/,
	);
});

test('harper-editor becomes an inert placeholder', () => {
	const content = render('```harper-editor\nLine one\nline <two>\n\nSecond paragraph\n```');

	assert.match(
		content,
		/<!-- wp:html -->\n<div data-docs-embed="editor" data-harper-editor><p>Line one<br \/>line &lt;two&gt;<\/p><p>Second paragraph<\/p><\/div>/,
	);
});

test('harper-rules becomes the generated table', () => {
	const rules = {
		descriptions: { SpellCheck: '<p>Finds <code>mispellings</code>.</p>', Off: '<p>Off.</p>' },
		defaultConfig: { SpellCheck: true, Off: false },
	};
	const content = render('```harper-rules\n```', {}, { rules });

	assert.match(
		content,
		/<tr><td>SpellCheck<\/td><td>✔️<\/td><td>Finds <code>mispellings<\/code>.<\/td><\/tr>/,
	);
	assert.match(content, /<tr><td>Off<\/td><td>❌<\/td><td>Off.<\/td><\/tr>/);
	assert.match(problems({ 'docs/about.md': page('About', '```harper-rules\n```') })[0], /rules/);
});

test('links between pages become site URLs and are checked', () => {
	const content = render(
		'[First](./guides/first.md) and [a section](./guides/first.md#Getting-Started) and [source](../src/lib.rs) and [out](https://example.com/a) and [root](/wpdemo).',
		{ 'src/lib.rs': '' },
	);

	assert.match(content, /<a href="\/docs\/guides\/first">First<\/a>/);
	assert.match(content, /<a href="\/docs\/guides\/first#Getting-Started">/);
	assert.match(
		content,
		/<a href="https:\/\/github.com\/Automattic\/harper\/blob\/master\/src\/lib.rs">/,
	);
	assert.match(content, /<a href="https:\/\/example.com\/a">/);
	assert.match(content, /<a href="\/wpdemo">/);
});

test('broken links fail the build', () => {
	const broken = (body) => problems({ 'docs/about.md': page('About', body) })[0];

	assert.match(broken('[x](./guides/first.md#Nope)'), /no heading with that id/);
	assert.match(broken('[x](./missing.md)'), /does not exist/);
	assert.match(broken('[x](#Nope)'), /no heading with that id/);
	assert.match(broken('[x](javascript:alert(1))'), /not allowed/);
	assert.match(broken('![x](/images/missing.png)'), /image \/images\/missing.png not found/);
});

test('block attributes cannot close the comment', () => {
	assert.equal(
		blockAttributes({ a: '-->', b: '<"&>' }),
		'{"a":"\\u002d\\u002d\\u003e","b":"\\u003c\\u0022\\u0026\\u003e"}',
	);
});
