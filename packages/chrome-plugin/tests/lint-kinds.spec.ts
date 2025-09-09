import { expect, test } from '@playwright/test';
import { LINT_KINDS, lintKindColor } from 'lint-framework';

test('display lint kind colors', async ({ page }, testInfo) => {
	// Generate color boxes for each lint kind
	const colorBoxes = LINT_KINDS.map((kind) => {
		const color = lintKindColor(kind);
		return `<div class="color-box" style="background-color: ${color}">${kind}</div>`;
	}).join('\n');

	const htmlContent = `
    <!DOCTYPE html>
    <html>
    <head>
      <title>Lint Kind Colors</title>
      <style>
        body {
          font-family: Arial, sans-serif;
          padding: 20px;
          background: #f5f5f5;
        }
        h1 {
          color: #333;
          margin-top: 0;
        }
        .container {
          display: flex;
          flex-wrap: wrap;
          gap: 10px;
          margin-top: 20px;
        }
        .color-box {
          padding: 10px 20px;
          border-radius: 4px;
          color: white;
          font-weight: bold;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
      </style>
    </head>
    <body>
      <h1>Lint Kind Colors</h1>
      <div class="container">
        ${colorBoxes}
      </div>
    </body>
    </html>
  `;

	// Attach the HTML report
	await testInfo.attach('lint-colors.html', {
		body: htmlContent,
		contentType: 'text/html',
	});
});
