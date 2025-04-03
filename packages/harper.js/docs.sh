#! /bin/bash

set -eo pipefail

pnpm api-extractor run 
pnpm api-documenter markdown -i temp

rm -r html || true
mkdir html || true

echo Rendering HTML...

parallel '
  BASE=$(basename {} .md)
  pandoc {} -o html/$BASE.html
  perl -pi -e "s/\"\\K([^\"]+)\.md(?=\")/\1.html/g" html/$BASE.html
  echo "<link rel=\"stylesheet\" href=\"https://unpkg.com/mvp.css\">" >> "html/$BASE.html"
' ::: ./markdown/*.md

rm -r ../web/static/docs/harperjs || true
mkdir -p ../web/static/docs/harperjs || true
mv -f html ../web/static/docs/harperjs/ref 
