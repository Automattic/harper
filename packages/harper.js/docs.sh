#!/usr/bin/env bash

set -eo pipefail

pnpm api-extractor run
pnpm api-documenter markdown -i temp

html_dir="./html"
if [[ -d "$html_dir" ]]; then
	echo "Deleting old output from ${html_dir}"
	rm -r "$html_dir" || true
fi
mkdir "$html_dir" || true

harperjs_docs_dir="../web/static/docs/harperjs"
if [[ -d "$harperjs_docs_dir" ]]; then
	echo "Deleting old output from ${harperjs_docs_dir}"
	rm -r "$harperjs_docs_dir" || true
fi
mkdir -p "$harperjs_docs_dir" || true

echo "Rendering HTML..."
if command -v parallel &> /dev/null; then
	parallel '
        base=$(basename {} .md)
        node renderPage.js "${base#"harper.js."} - Harper" "API reference documentation for harper.js" {} "html/${base}.html"
    ' ::: ./markdown/*.md
else
	echo "parallel not found, falling back to sequential processing"
	for file in ./markdown/*.md; do
		base=$(basename "$file" .md)
    node renderPage.js "${base#"harper.js."} - Harper" "API reference documentation for harper.js" {} "html/${base}.html"
	done
fi
mv -f "$html_dir" "${harperjs_docs_dir}/ref"
