#!/bin/sh
# Inside the development runner: a copy of the hive's tree in /kind, from which the judge runs (the hive's tree is
# mounted read-only on /hive). Then the command given, if any.
set -eu
rm -rf /kind
mkdir -p /kind
cp -r /hive/. /kind/
[ $# -eq 0 ] || exec "$@"
