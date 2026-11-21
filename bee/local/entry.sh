#!/bin/sh
# Entry of the local runner (bee/local/Dockerfile): starts this container's own Docker engine, waits for it, then
# runs the command it was given.
set -eu

# cgroup v2: the engine's containers need the controllers (memory, pids) delegated to them. Processes of a cgroup
# that delegates must sit in a leaf: move them there first.
if [ -f /sys/fs/cgroup/cgroup.controllers ]; then
  mkdir -p /sys/fs/cgroup/init
  xargs -rn1 < /sys/fs/cgroup/cgroup.procs > /sys/fs/cgroup/init/cgroup.procs 2>/dev/null || :
  sed -e 's/ / +/g' -e 's/^/+/' < /sys/fs/cgroup/cgroup.controllers > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || :
fi

dockerd > /var/log/dockerd.log 2>&1 &
tries=0
until docker info > /dev/null 2>&1; do
  tries=$((tries + 1))
  if [ "$tries" -gt 90 ]; then
    echo "entry: the nested Docker engine did not start:" >&2
    tail -n 40 /var/log/dockerd.log >&2
    exit 97
  fi
  sleep 1
done
exec "$@"
