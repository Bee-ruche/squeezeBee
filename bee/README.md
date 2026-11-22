# The project's tooling (`bee/`)

Everything here is the project's own: no contribution can change it, and the judge takes it from the project's
tip, never from a candidate.

| Path | What |
| --- | --- |
| `corpus/` | The corpus (`data`, 10,000,000 bytes of public-domain books) and `make.py`, the rule it was made by |
| `relay/project.json` | The judge's manifest: the corpus (its path and its SHA-256), what a build is given, the program's paths, the bounds |
| `relay/build` | The build, run by the judge in a container: the rule, then the compiler, then the frame; it leaves one file, `run` |
| `relay/rules.py` | The rule on what a file of the program may hold (AGENTS.md), and the crate root the build compiles |
| `relay/main.rs` | The frame around the program: the only code that touches a file |
| `sandbox/Dockerfile`, `sandbox/sans-docker.sh`, `check` | A worker's sandbox, with Docker and without, and its quick check; `sandbox/test_sandbox.sh`, the proofs of the launcher |
| `proofs/` | Candidates, honest and hostile, and `prove.py`, which has the judge answer each and checks the answers |
| `local/` | The judge on a machine that has Docker and no Python: one privileged container that holds its own Docker engine (never for a volunteer's candidate) |

## The judge

The project is judged by the hive's third kind of judge, `relay` (`tools/judge/relay.py` and `docs/relay-judge.md` in
Bee-ruche/hive, at the commit `.github/workflows/hive.yml` names). In short, for a candidate and the project's tip:

1. the candidate's tree must be the tip's everywhere but under `src/`;
2. both programs are built by `relay/build`, each in a container that holds this folder's skeleton and one `src/`,
   and never the corpus;
3. each program encodes the corpus twice, in containers of their own, and each one's two messages must be the same
   bytes; the tip's decodes once and the candidate's twice, and each result must be the corpus, byte for byte;
4. every phase has a world of its own: a memory file system made for it, which holds its own copies of the
   launcher, the gate, the program and its one input (the decode phase's is the message), gone when it ends;
5. `total_bytes` is the message's length plus the length of the built `run`, both counted by the judge.

Each phase has one process, no network, the manifest's processor time (the kernel's own count, read by the judge's
launcher) and the manifest's memory. A phase's container is built on an **empty image**: no system, no library,
no file but what the judge puts there. That is why `relay/build` links `run` statically: the program brings what
it needs, and all of it is counted. And the judge takes `run` for native code that could make any system call,
whatever its source says: its gate opens the two files of the phase (the program reads descriptor 3 and writes
descriptor 4, which is all `relay/main.rs` does with files), then shuts every call but reading and writing those,
memory and ending, and only then starts the program.

## Running the judge yourself

On Linux with Docker, Python 3 and the right to mount (root, or `sudo -n`), from a tree that holds the hive's
`tools/judge/` with `relay.py` in it:

```bash
python3 tools/judge/relay.py --tip <the project's tree> --image-only                  # once, with network: the image, the corpus
python3 tools/judge/relay.py --tip <the project's tree> --tip-itself --json out.json   # the tip's own figures
python3 tools/judge/relay.py --tip <the project's tree> --candidate <a candidate's tree> --json out.json
```

Both trees are plain folders with no `.git` in them (`git archive <commit> | tar -x -C <folder>`). The exit code is
0 (right: the figures are in the report), 1 (refused: the reason is in the report), 2 (the judge could not say).

On a machine without Python: `local/` builds a runner, one container that holds a Docker engine
of its own, so that every container the judge starts is nested in it and bounded with it. It is privileged: **never
run a volunteer's candidate with it** ([local/README.md](local/README.md)).

```bash
docker build -t bee-relay-local bee/local                         # once, with network (Ubuntu's packages)
bee/local/run.sh [--network] <the hive's tree> <command…>         # the project on /project, the hive on /hive, read-only
```

The hive's tree is Bee-ruche/hive at the commit `.github/workflows/hive.yml` names, written out (`git archive
<commit> | tar -x -C <folder>`). `bee/local/assemble.sh`, inside the runner, copies it to `/kind`, from which the
judge runs. The runner is given the network for the image step alone; without `--network` neither it nor anything
in it has any.

## The proofs

`proofs/prove.py` judges the tip against itself and each candidate of `proofs/`, and ends with 0 only if every
answer is the expected one ([proofs/README.md](proofs/README.md)). It judges on the project's own corpus; with
`--bytes N`, on a stand-in of N bytes instead (`local/standin.py`: text made from the hive's own files, seeded),
for a timing at another size.

```bash
# in the runner: the tip and the honest candidate, with the manifest's own bounds
bee/local/run.sh <hive> sh /project/bee/local/assemble.sh python3 -B /project/bee/proofs/prove.py \
  --kind /kind/tools/judge --hive /hive --project /project --work /work tip honest-order3
# every proof, with bounds lowered so that a bound is reached in seconds and within the runner's memory
bee/local/run.sh <hive> sh /project/bee/local/assemble.sh python3 -B /project/bee/proofs/prove.py \
  --kind /kind/tools/judge --hive /hive --project /project --work /work \
  --encode-seconds 30 --decode-seconds 30 --memory-mb 512
```

## The sandbox

Two forms, whose commands AGENTS.md gives ("Commands"): the image (`sandbox/Dockerfile`) where the machine has
Docker, and `sandbox/sans-docker.sh <command>` on Linux where it has not. Where neither works, a worker stops.

**The image** is a container without network, without capability, as an unprivileged user, that sees the clone and
nothing else of the machine, with `.git`, `.github`, the agents' instructions and `bee/` read-only. It keeps a
program that goes wrong from the machine and from the network; it does not make the clone trustworthy again: after
project code ran there, treat the work tree as untrusted (the hive's own commands check it before anything is
pushed).

**Without Docker**, `sandbox/sans-docker.sh` is the launcher of the hive's two other pilots, word for word but for
the paths it guards: all of `bee/` here, where they guard `bee/sandbox` alone, so that it holds what this project's
image holds read-only (and `bee/local/run.sh`, which runs on a machine with a privileged container, is not the
command's to change). Inside it, a command:

- has **no network** (an empty network namespace) and **none of your environment variables** (no token, no proxy):
  only `PATH`, `HOME`, `LANG`, `TERM`;
- can **write** in the work tree, which lasts, and in a private `/tmp` (with its empty home, `/tmp/home`) and a
  private `/dev/shm`, which are gone when the command ends; nowhere else;
- **is shown of `.git` only the history**: `HEAD`, `refs`, `packed-refs`, `objects`, `shallow` and the index,
  read-only, with empty settings. The clone's settings (the remotes' addresses, a credential if one was written
  there), its logs, its hooks and whatever else `.git` holds are not shown. Git reads the history inside, and writes
  nothing. The launcher refuses to run if `.git` is not a real folder in git's plain format, if its settings are not
  a plain file, if one of those entries is a link, if a folder under them cannot be listed, or if a file under them
  is a second name of a file elsewhere (a pack shared with another clone is fine);
- **cannot change the guarded paths** of the work tree: `.git`, `.github`, `.claude`, `.mcp.json`, `AGENTS.md`,
  `CLAUDE.md`, `bee` (the project's tooling, this launcher among it). They are read-only and can be neither renamed
  nor removed. One that does not exist cannot be covered: if the command creates it, the launcher renames it
  `<name>.created-in-sandbox.XXXXXX` when the command ends, and says so (exit status 125 if it cannot; a launcher
  killed outright renames nothing: look at the tree before you use it);
- **does not see the rest of the machine**: your home, the folders next to the clone, `/var`, `/run` and its sockets
  (Docker, D-Bus, ssh-agent), `/sys`, the disks and terminals of `/dev`, the other processes, the descriptors its
  caller left open. `/usr`, `/bin`, `/sbin`, `/lib*` and `/etc` are there read-only; in `/etc`, what "others" cannot
  read (`shadow`, private keys) is hidden, also when the session is root;
- **cannot change what its caller hands it** as input and outputs (a file given as input or output keeps its mode
  and dates; the devices it is shown work and are read-only as files of the machine);
- **leaves no core file** when it crashes, and **no process behind** (a PID namespace, killed when the command ends,
  when the launcher is signalled, or when it is killed);
- **cannot undo any of this**: it has no capability and can regain none; it can make no user namespace of its own.

If one of these protections cannot be set up, the launcher does not run the command: exit status 125 and the reason
on stderr. Otherwise stdin, stdout, stderr and the exit status are the command's (128+N if signal N killed it).
`sandbox/test_sandbox.sh sandbox/sans-docker.sh <an empty folder>` checks each of them on a throwaway repository it
makes there (read its first lines before running it as root): the first pilot's script, with this project's
guarded `bee/` in its fixture.

For this project's commands it changes little: `bee/check` reads the clone and writes in the private `/tmp` only.
A rename or a hard link between `bee/` and the rest of the tree fails ("cross-device"); the command is root in name
only (no capability: `mount` and `unshare -r` fail).

What it does not guarantee:

- **It is not a virtual machine.** The command talks to the kernel of your machine: a kernel vulnerability or a
  processor's side channel crosses it.
- **The work tree stays writable**: code that runs inside can change any file of the tree but the guarded ones, and
  leave a file under any other name. After a run, treat the tree as untrusted and execute nothing of it outside the
  sandbox. A worker runs no git command in the clone: `hive submit` checks the clone and commits the task's files
  only.
- **It can read** the work tree and the history (keep no secret in either), `/usr`, the world-readable part of
  `/etc`, and what `/proc` says of the machine (processor, memory, kernel). The Docker form shows `.git` whole,
  read-only: there, keep no credential in `.git/config`.
- **No limit on processor time, memory, processes or disk.** A command can fill the disk through the work tree or
  the memory through `/tmp`. Your own timeout ends it: killing the launcher kills everything inside.
- **It shows the machine's own `/usr`**, whatever compiler is there: what `bee/check` does about it is below.

**The toolchain without Docker.** The judge builds the program in its image: Ubuntu 24.04 (by its fingerprint),
that release's rustc 1.75 for x86-64 (`x86_64-unknown-linux-gnu`), linked by gcc, statically, against the release's
own C library (`libc.a`, of `libc6-dev`), with python3 for the project's rule; nothing is fetched while it builds.
`program_bytes` is the length of that file, so it depends on the compiler's release, the C library and the linker
(most of the frame's 1.3 MB is theirs). `message_bytes` depends on the program's arithmetic alone (whole numbers and
the four operations of floating point), the same on any compiler and any processor. The launcher shows whatever
compiler the machine has, so **`bee/check` checks with the judge's toolchain or not at all**: before it builds, it
stops (exit status 3) when `rustc` is not 1.75, when there is no `cc` or no static C library, or no `python3`, and
says which, and what to do. It does not go on with another compiler and a warning: a later release builds what the
judge's refuses (a part of the language stabilised after 1.75), which the judge answers with a refusal the
worker's deal pays for, and `program_bytes` is part of the very total a task's goal is on. The one difference it
lets through is the processor's: on ARM the compiler is the same release, so what builds there builds at the judge;
it then says that `program_bytes` is not the judge's (`message_bytes` is).

**What a machine without Docker has** cannot be known from here, a cloud session's least of all: whether it lets a
process make user namespaces (if not, the launcher says so and stops), which system and which compiler it holds,
whether it reaches Ubuntu's archive. On Ubuntu 24.04 that reaches it, the release's own packages are the judge's
compiler (`apt-get install rustc gcc libc6-dev python3-minimal`, before any code of the project runs: AGENTS.md); a
later update of a package may move `program_bytes` by some bytes, as it may on the judge's machine (DESIGN.md, "The
compiler is not pinned"). Anywhere else, **a worker without Docker and without this compiler cannot check, and
stops.** For it to be otherwise, the project and the hive would need a toolchain that such a machine can fetch and
hold to a fingerprint, and that the judge builds with too: for instance rustc 1.75.0 from the Rust project's own
release archives, pinned by SHA-256 in the hive's image and in `bee/check`, and a C library that comes with it
(Rust's musl target carries its own) or none to speak of (a frame without the standard library, the architect's
own direction). That changes the hive's image and this project's build: not made here.

**Tried** (22 November 2026), in a throwaway container of the judge's Ubuntu 24.04 holding the packages AGENTS.md
names and git and python3 (kernel 6.18, util-linux 2.39.3, git 2.43), without network, with Docker's system-call
filter and its masking of `/proc` lifted so that user namespaces work there as on a machine without Docker:

- `test_sandbox.sh`, as root: 84 checks passed, 1 skipped (no disk device in a container); as an unprivileged user:
  78 passed, 7 skipped (what that user cannot do anyway). One failed both ways, `g2`: it fakes a masked `/proc` with
  a mount made in the caller's own user namespace, and this kernel lets a namespace made below it mount a fresh
  `/proc` all the same. The first pilot's own launcher and script fail it there in the same way. The real case it
  stands for, Docker's own masked `/proc`, was tried: the launcher refuses (exit status 125) and runs nothing.
- Inside the launcher, as root and as that user, `bee/check` and `bee/check 0` gave the judge's own figures (on the
  whole corpus: `program_bytes` 1,346,472, `message_bytes` 3,544,012) and left the clone as it was. With another
  release of `rustc` first on the PATH, without the static C library, without `cc`, and with the judge's own
  compiler in the home folder only, it stopped (exit status 3) and said why.
- Not tried: a cloud session (whether it lets a process make user namespaces, and reaches Ubuntu's archive), ARM.

In the sandbox the program runs without the judge's gate: `check` opens its two files with the shell
(`run encode 3<data 4>message`). The language and the rule keep it from opening anything else (AGENTS.md, "The
rules of the program"); the sandbox is there for the day that rule, or the compiler, is wrong. `check` never goes
through the gate, on x86-64 or on ARM (where it should run, not tried): its times are never the judge's, its program
size is not on ARM, and a program that reads a fine clock or makes a call the gate refuses passes it and dies at
the judge (AGENTS.md, "Commands").
