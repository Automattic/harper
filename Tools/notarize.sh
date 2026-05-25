#!/usr/bin/env bash

set -euo pipefail

usage() {
	cat <<'EOF'
Usage:
  Tools/notarize.sh <path>

Submits <path> (an .app bundle or .dmg) to Apple's notary service with
`xcrun notarytool`, waits for the verdict, fetches the log on failure,
and staples the ticket to <path> on success.

Auth — App Store Connect API key from env (matches the Fastlane env var
convention used by CI):

  APP_STORE_CONNECT_API_KEY_KEY_ID
  APP_STORE_CONNECT_API_KEY_ISSUER_ID
  APP_STORE_CONNECT_API_KEY_KEY            (PEM; \n escapes decoded)
EOF
}

target=""

while [ "$#" -gt 0 ]; do
	case "$1" in
		-h|--help)
			usage
			exit 0
			;;
		-*)
			echo >&2 "unknown option: $1"
			usage >&2
			exit 1
			;;
		*)
			if [ -n "$target" ]; then
				echo >&2 "unexpected extra positional argument: $1"
				usage >&2
				exit 1
			fi
			target="$1"
			shift
			;;
	esac
done

[ -n "$target" ] || { echo >&2 "missing target path"; usage >&2; exit 1; }
[ -e "$target" ] || { echo >&2 "target does not exist: $target"; exit 1; }

key_id="${APP_STORE_CONNECT_API_KEY_KEY_ID-}"
issuer_id="${APP_STORE_CONNECT_API_KEY_ISSUER_ID-}"
key_pem="${APP_STORE_CONNECT_API_KEY_KEY-}"

if [ -z "$key_id" ] || [ -z "$issuer_id" ] || [ -z "$key_pem" ]; then
	echo >&2 "missing App Store Connect API key env vars"
	echo >&2 "set APP_STORE_CONNECT_API_KEY_{KEY_ID,ISSUER_ID,KEY}"
	exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

p8="$work/AuthKey_${key_id}.p8"
# %b decodes literal `\n` escapes into real newlines. Trailing newline
# is mandatory: notarytool's PEM parser rejects keys without one as
# `invalidPEMDocument` (openssl is more lenient).
printf '%b\n' "$key_pem" > "$p8"
chmod 600 "$p8"

# notarytool accepts .dmg/.pkg/.zip directly; .app bundles must be zipped.
case "$target" in
	*.app|*.app/)
		submit_path="$work/$(basename "$target").zip"
		echo "==> zipping app for submission"
		ditto -c -k --sequesterRsrc --keepParent "$target" "$submit_path"
		;;
	*.dmg|*.pkg|*.zip)
		submit_path="$target"
		;;
	*)
		echo >&2 "unsupported target type: $target"
		exit 1
		;;
esac

echo "==> submitting to notarytool (this can take a few minutes)"
submit_json="$work/submit.json"
xcrun notarytool submit "$submit_path" \
	--key "$p8" \
	--key-id "$key_id" \
	--issuer "$issuer_id" \
	--wait \
	--output-format json \
	> "$submit_json"

cat "$submit_json"
echo

status="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"])' "$submit_json")"
submission_id="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["id"])' "$submit_json")"

if [ "$status" != "Accepted" ]; then
	echo >&2 "==> notarization status: $status — fetching log"
	xcrun notarytool log "$submission_id" \
		--key "$p8" --key-id "$key_id" --issuer "$issuer_id"
	exit 1
fi

echo "==> notarization accepted (id=$submission_id)"

echo "==> stapling $target"
xcrun stapler staple "$target"
xcrun stapler validate "$target"

echo "==> done"
