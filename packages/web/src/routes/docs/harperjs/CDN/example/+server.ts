import pageHtml from '../../../../../../../harper.js/examples/raw-web/index.html?raw';

export async function GET() {
	return new Response(pageHtml, {
		headers: {
			'Content-Type': 'text/html',
		},
	});
}
