# A local runner for the judge on a machine that has Docker and no Python (bee/README.md): ONE container that
# holds a Docker engine of its own, the judge's Python and nothing of the project. The judge runs inside it as it
# runs on a hosted runner (root, Docker, mount); every container the judge starts is nested in this one, so the
# limits given to this one (--cpus, --memory) bound them all. It is a development tool: no run of the hive uses it.
#
#   docker build -t bee-relay-local bee/local                      (with network: Ubuntu's packages)
#   bee/local/run.sh --network <command>                           (see run.sh)
#
# The base image is the common judge's, by the same fingerprint (tools/judge/Dockerfile.base in Bee-ruche/hive).
FROM mirror.gcr.io/library/ubuntu@sha256:24737cef17f4e22b1977df7d9a05e77c4e28b7da5b8911f8d8ccda136bf3573c

RUN apt-get update -q \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y -q --no-install-recommends \
      docker.io docker-buildx iptables ca-certificates python3 git \
 && rm -rf /var/lib/apt/lists/*

COPY entry.sh /usr/local/bin/entry.sh
ENTRYPOINT ["/bin/sh", "/usr/local/bin/entry.sh"]
