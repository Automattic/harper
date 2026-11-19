#!/usr/bin/env bash
# Downloads the prebuilt harper-ls for this Mac's architecture into vendor/harper-ls/.
set -euo pipefail
cd "$(dirname "$0")/.."

TAG="${1:-v2.13.0}"
case "$(uname -m)" in
  arm64)  TRIPLE="aarch64-apple-darwin" ;;
  x86_64) TRIPLE="x86_64-apple-darwin" ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

mkdir -p vendor/harper-ls
gh release download "$TAG" --repo Automattic/Harper \
  --pattern "harper-ls-$TRIPLE.tar.gz" --dir vendor/harper-ls --clobber
tar -xzf "vendor/harper-ls/harper-ls-$TRIPLE.tar.gz" -C vendor/harper-ls
xattr -d com.apple.quarantine vendor/harper-ls/harper-ls 2>/dev/null || true
vendor/harper-ls/harper-ls --version
