#!/usr/bin/env bash
# Runs one command in the local runner (bee/local/Dockerfile), from Git Bash or a Linux shell:
#
#   bee/local/run.sh [--network] [--seconds N] <hive clone> <command> [arguments…]
#
# What the container sees: this project's tree on /project and the hive's clone on /hive, both read-only; a scratch
# folder on /scratch if BEE_SCRATCH names one (read-write: reports, test trees); a Docker volume, bee-relay-docker,
# that keeps the nested engine's images between runs. Nothing else of the machine, and no token.
#   --network   leaves the container its network: ONLY for the image step (the judge's image, the pinned corpus).
#               Without it the container has none, and neither has anything the judge starts in it.
#   --seconds   the whole command's time (default 3600): the container ends itself after it.
# One container at a time, two processors, two gigabytes by default (BEE_CPUS, BEE_MEMORY): a small machine's limits.
#
# NEVER RUN A VOLUNTEER'S CANDIDATE WITH THIS. The container is privileged (it holds a Docker engine): whatever ran
# as root in it could reach the machine it runs on. It is for the project's own proof candidates (bee/local/README.md).
set -euo pipefail
network="--network none"
seconds=3600
while [ $# -gt 0 ]; do
  case "$1" in
    --network) network=""; shift ;;
    --seconds) seconds="$2"; shift 2 ;;
    *) break ;;
  esac
done
[ $# -ge 2 ] || { sed -n '2,13p' "$0" >&2; exit 2; }
# A folder's path as Docker wants it: Windows' own form under Git Bash (pwd -W), the plain one elsewhere.
folder() { (cd "$1" && { pwd -W 2>/dev/null || pwd; }); }
hive="$(folder "$1")"
shift
project="$(folder "$(dirname "$0")/../..")"
scratch=()
if [ -n "${BEE_SCRATCH:-}" ]; then
  scratch=(--mount "type=bind,source=$(folder "$BEE_SCRATCH"),target=/scratch")
fi
# shellcheck disable=SC2086
MSYS_NO_PATHCONV=1 exec docker run --rm --privileged --cpus "${BEE_CPUS:-2}" --memory "${BEE_MEMORY:-2g}" $network \
  --mount "type=volume,source=bee-relay-docker,target=/var/lib/docker" \
  --mount "type=bind,source=$project,target=/project,readonly" \
  --mount "type=bind,source=$hive,target=/hive,readonly" \
  "${scratch[@]}" \
  -e BEE_JUDGE_INTEGRATION="${BEE_JUDGE_INTEGRATION:-}" -e BEE_JUDGE_NETWORK="${BEE_JUDGE_NETWORK:-host}" \
  bee-relay-local timeout -s KILL "$seconds" "$@"
