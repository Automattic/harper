#!/usr/bin/env bash
# Install system packages with apt, giving up quickly when Ubuntu's mirrors
# are unreachable instead of hanging until the job's six-hour timeout.
#
# Usage: .github/scripts/apt-install.sh <package>...

set -euo pipefail

# Both commands normally finish in under a minute.
limit=300
opts=(-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30)

fail() {
	echo "::error title=Could not install system packages::apt-get failed or took longer than ${limit}s. This is usually an Ubuntu mirror or DNS outage rather than a problem with this change. Check https://status.canonical.com and https://www.githubstatus.com, then re-run the job."
	exit 1
}

timeout "$limit" sudo apt-get "${opts[@]}" update || fail
timeout "$limit" sudo apt-get "${opts[@]}" install -y "$@" || fail
