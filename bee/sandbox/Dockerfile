# The project's sandbox: where a worker builds and checks the program, without network and with nothing of its
# machine but the project's clone (bee/README.md).
#
#   docker build -t bee-squeeze bee/sandbox
#
# It is, instruction for instruction, the image the hive's judge builds for this project (tools/judge/Dockerfile.base
# and the packages of its « rust » image, in Bee-ruche/hive): the same Ubuntu by its fingerprint, the same compiler
# as that release packages it (rustc 1.75), nothing fetched when a program is built. The corpus is in the clone
# (bee/corpus/data): the image holds none, and building it fetches nothing but Ubuntu's packages.
FROM mirror.gcr.io/library/ubuntu@sha256:24737cef17f4e22b1977df7d9a05e77c4e28b7da5b8911f8d8ccda136bf3573c

RUN apt-get update -q \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y -q --no-install-recommends \
      gcc make binutils libc6-dev valgrind python3-minimal \
 && rm -rf /var/lib/apt/lists/*

RUN ln -s / /.root

RUN apt-get update -q \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y -q --no-install-recommends \
      rustc lld \
 && rm -rf /var/lib/apt/lists/*

# An unprivileged user; uid 1000 is the first user of most Linux machines, which keeps what is written in the
# clone readable outside the container.
RUN userdel -r ubuntu 2>/dev/null || true \
 && useradd --create-home --uid 1000 --shell /bin/bash bee
USER bee
WORKDIR /work
CMD ["bash"]

