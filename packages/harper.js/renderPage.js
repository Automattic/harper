import fs from 'fs';
import { marked } from 'marked';

const pageTitle = process.argv[2];
const description = process.argv[3];
const input = process.argv[4];
const output = process.argv[5];

const renderer = new marked.Renderer();

renderer.link = ({ href, title, text }) => {
	if (href.endsWith('.md')) {
		href = `${href.slice(0, href.length - 3)}.html`;
	}
	const titleAttr = title ? ` title="${title}"` : '';
	return `<a href="${href}" ${titleAttr}>${text}</a>`;
};

const markdown = fs.readFileSync(input, 'utf8');
const body = marked.parse(markdown, { async: false, renderer });

const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>${pageTitle}</title>
<meta name="description" content="${description}">
<link
  rel="stylesheet"
  href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css"
>
</head>
<body>
${body}
</body>
</html>
`;

fs.writeFileSync(output, html);
