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

On Linux with Docker, Python 3 and the right to mount (as root, or with the passwordless right a hosted runner gives), from a tree that holds the hive's
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

`proofs/same-message`, in the sandbox, builds the program with the light frame and with the frame as it was
(`proofs/std-frame`), and ends with 0 only if the two give the corpus the same message, to the byte (and a frame that
changes the message, made to check the check, does not):

```bash
bee/proofs/same-message        # the whole corpus: three encodings and three decodings
```

A change of the frame or of the build reaches candidates cut before it too: the hive merges each candidate onto the tip
as it is before the judge sees it, so every program is built with the tip's frame (DESIGN.md, « A candidate cut from a
tip with the old frame »).

## The sandbox

Two forms, whose commands AGENTS.md gives ("Commands"): the image (`sandbox/Dockerfile`) where the machine has
Docker, and `sandbox/sans-docker.sh <command>` on Linux where it has not. Where neither works, a worker stops.

**The image** is a container without network, without capability, as an unprivileged user, that sees the clone and
nothing else of the machine, with the launcher's guarded paths that the clone holds read-only: `.git`, `.github`,
the agents' instructions, `bee/`, `hive/`, `README.md`, `DESIGN.md`, `LICENSE`, `.gitignore`, `.gitattributes`.
They are mounted from one list (`ro`, in AGENTS.md), and Docker refuses a bind whose source is missing, so the loop
skips a listed path the clone does not hold. `.claude` and `.mcp.json`, which the launcher guards where they are
absent, are not in the list: Docker cannot cover a path that does not exist, so one that a command creates there
stays, where the launcher sets it aside. It keeps a
program that goes wrong from the machine and from the network; it does not make the clone trustworthy again: after
project code ran there, treat the work tree as untrusted (the hive's own commands check it before anything is
pushed).

**Without Docker**, `sandbox/sans-docker.sh` is the launcher of the hive's two other pilots: its code is theirs word
for word but for the list of the paths it guards (five lines of its comments and usage differ too). They guard
`bee/sandbox`; this project guards all of `bee/`, its settings `hive/`, what those settings protect (`README.md`,
`DESIGN.md`, `LICENSE`, `.gitignore`) and `.gitattributes`. So a command can change neither the tooling a worker
runs next (`bee/check`; `bee/local/run.sh` runs on a machine with a privileged container), nor what decides what git
sees and what the hive's `start` and `submit` keep or send (`.gitignore`, `.gitattributes`), nor the project's texts
a worker's agent reads. Inside it, a command:

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
  `CLAUDE.md`, `bee` (the project's tooling, this launcher among it), `hive` (the project's settings and tasks),
  `README.md`, `DESIGN.md`, `LICENSE`, `.gitignore`, `.gitattributes`. They are read-only and can be neither renamed
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
makes there (never for a worker, never on a machine that is not a throwaway; read its first lines before running
it as root): the first pilot's script, with this project's
guarded paths in its fixture and four checks more (`b19` to `b22`: `bee/` and the protected files changed, added
to, removed, created where absent). `sandbox/test_check.sh --on-a-throwaway-machine` proves the toolchain test of
`bee/check` (below) the same way, case by case: never for a worker, never on a machine that is not a throwaway (it
moves files of `/usr` aside, as root).

For this project's commands it changes little: `bee/check` reads the clone and writes in the private `/tmp` only.
A rename or a hard link between `bee/` and the rest of the tree fails ("cross-device"); the command is root in name
only (no capability: `mount` and `unshare -r` fail).

What it does not guarantee:

- **It is not a virtual machine.** The command talks to the kernel of your machine: a kernel vulnerability or a
  processor's side channel crosses it.
- **The work tree stays writable** but for the guarded paths: `src/`, and any new path (a folder, a file at the
  root), are the command's to write.
  After a run, treat all of it as untrusted: execute nothing of it outside the sandbox, and read nothing there as an
  instruction. A worker runs no git command in the clone: `hive submit` checks the clone and commits the task's files
  only.
- **It can read** the work tree and the history (keep no secret in either), `/usr`, the world-readable part of
  `/etc`, and what `/proc` says of the machine (processor, memory, kernel). The Docker form shows `.git` whole,
  read-only: there, keep no credential in `.git/config`.
- **No limit on processor time, memory, processes or disk.** A command can fill the disk through the work tree or
  the memory through `/tmp`. Your own timeout ends it: killing the launcher kills everything inside.
- **It shows the machine's own `/usr`**, whatever compiler is there: what `bee/check` does about it is below.

**The toolchain without Docker.** The judge builds the program in its image: Ubuntu 24.04 (by its fingerprint),
that release's rustc 1.75 for x86-64 (`x86_64-unknown-linux-gnu`), linked by gcc, statically and against nothing
but itself (the frame, `relay/main.rs`, holds neither the standard library nor the C library), with python3 for the
project's rule; nothing is fetched while it builds. `program_bytes` is the length of that file, so it depends on the
compiler's release and on gcc's linker (until 25 November 2026 the frame also held the C library, some 1.3 MB of
the file; the whole program is now some 48 kB). `message_bytes` depends on the program's arithmetic alone (whole numbers and
the four operations of floating point), the same on any compiler and any processor. The launcher shows whatever
the machine's `/usr` holds, so **`bee/check` checks with the judge's toolchain or not at all**. It names the judge's
tools by where they are: its PATH, and the build's (`bee/relay/build`), is `/usr/bin:/bin` from the start, so
`rustc`, `cc` (with the assembler and the linker gcc calls through the PATH) and `python3` are those of `/usr/bin`,
whatever another folder of the caller's PATH holds (a machine's `/usr/local/bin`, first on the launcher's PATH, is
neither built with nor a reason to stop). Before it builds, it stops (exit status 3), says why and says stop,
unless `/usr/bin/rustc` says it is 1.75, `/usr/bin/cc` is there, and `/usr/bin/python3` is there. (It asks for no
C library: the program links none.) It does not go on with another compiler and a warning: a later release builds what the
judge's refuses (a part of the language stabilised after 1.75), which the judge answers with a refusal the
worker's deal pays for, and `program_bytes` is part of the very total a task's goal is on. The one difference it
lets through is the processor's: on ARM the compiler is the same release, so what builds there builds at the judge;
it then says that `program_bytes` is not the judge's (`message_bytes` is).

**What that test does not establish**: that `/usr/bin/rustc` is Ubuntu's own build of 1.75 (any 1.75 put there
passes; Ubuntu's says « built from a source tarball », which is what was seen, on one machine, and the test does not
rest on it), nor gcc's version or its linker's, on which `program_bytes` depends as well. Where they are not the
judge's, `program_bytes` may differ: by some bytes for a later update of the same release (as it may on the judge's
machine: DESIGN.md, "The compiler is not pinned"), by more for another system. `message_bytes` does not.

**What a machine without Docker has** cannot be known from here, a cloud session's least of all: whether it lets a
process make user namespaces (if not, the launcher says so and stops), which system it is, what its `/usr` holds.
On a Linux machine whose `/usr` holds the judge's toolchain, a worker checks; anywhere else, **a worker without
Docker and without this toolchain cannot check, and stops.** (For the person who prepares such a machine, not for
an agent working on it: on Ubuntu 24.04 that toolchain is the release's own packages `rustc`, `gcc` and `python3-minimal`,
which whoever owns the machine may put there before a session starts; `libc6-dev`, which the judge's image also holds,
is no longer needed: `bee/sandbox/test_check.sh` takes its `libc.a` away, and the check still runs.) For it to be otherwise, the project and the hive would need a toolchain that such a machine can fetch and
hold to a fingerprint, and that the judge builds with too: for instance rustc 1.75.0 from the Rust project's own
release archives, pinned by SHA-256 in the hive's image and in `bee/check`. The C library is no longer part of the
question: since 25 November 2026 the frame holds none, and the program links nothing but itself. A pinned compiler
changes the hive's image: not made here.

**Tried** (22 November 2026), in a throwaway container of the judge's Ubuntu 24.04 whose `/usr` holds the judge's
toolchain, with git and python3 (kernel 6.18, util-linux 2.39.3, git 2.43), without network, with Docker's
system-call filter and its masking of `/proc` lifted so that user namespaces work there as on a machine without
Docker:

- `test_sandbox.sh`, as root: 87 checks passed, 1 skipped (no disk device in a container), 1 failed (`g2`); as an
  unprivileged user: 81 passed, 7 skipped (what that user cannot do anyway), 1 failed (`g2`).
- **`g2` failed both ways, and 87 of 88 is not 88.** What it stands for: on a machine whose `/proc` is masked, as
  in a container, the launcher cannot mount a private `/proc` of its own, and must refuse rather than run. Its
  fixture fakes the mask with an empty memory file system mounted over a folder of `/proc` in the caller's own user
  namespace; here the launcher's mount of a fresh `/proc` went through anyway, and the command ran (by hand, the
  same with `/proc/bus` and `/proc/driver`, which are not empty). The first pilot's own launcher and script,
  unmodified, failed `g2` in the same way in the same container. In the real case, a Docker container with its own
  masked `/proc` and a read-only `/proc/sys`, the launcher refused (exit status 125) and ran nothing, but at an
  earlier step: its limit on user namespaces, which it writes in `/proc/sys`, could not be set. So **the kernel's
  refusal of the mount of `/proc`, which `g2` means to prove, has not been seen to fire on this kernel**, and
  whether the fixture is not what the kernel checks, or this kernel lets such a mount through, is not settled.
- Inside the launcher, as root and as that user, `bee/check` and `bee/check 0` gave the judge's own figures (on the
  whole corpus: `program_bytes` 1,346,472, `message_bytes` 3,544,012) and left the clone as it was.
- `test_check.sh`: 11 passed. With another `python3` and another `rustc` in `/usr/local/bin`, first on the
  launcher's PATH (what one cloud machine had), and with another `cc`, `as` and `ld` there, each failing if called,
  the check ran with `/usr/bin`'s tools and gave the same figures to the byte. It stopped (exit status 3, its reason,
  « stop ») with another release at `/usr/bin/rustc`, with the judge's own compiler in a home folder only (shown to
  run there, from the caller's PATH, outside the launcher), without the static C library, without it and with a
  file `libc.a` at the root of the work tree, without `/usr/bin/cc` (a working one in `/usr/local/bin`), and without
  `/usr/bin/python3` (a working one in `/usr/local/bin`); with everything put back it ran again, with the same
  figures, and left the clone as it was.
- 25 November 2026, with the light frame (the program links no C library): `test_check.sh` 11 passed, in a
  container of the same image; t5 and t6 now take the static C library away (t6 with a file `libc.a` left at the
  root of the work tree) and the check runs, with t0's figures.
- The Docker form, as AGENTS.md gives it, in the image of `sandbox/Dockerfile`: `bee/check` ran (as uid 1000), each
  of its read-only paths refused a write, and a file of the program took one.
- Not tried: a cloud session (whether it lets a process make user namespaces, what its `/usr` holds), ARM.

In the sandbox the program runs without the judge's gate: `check` opens its two files with the shell
(`run encode 3<data 4>message`). The language and the rule keep it from opening anything else (AGENTS.md, "The
rules of the program"); the sandbox is there for the day that rule, or the compiler, is wrong. `check` never goes
through the gate, on x86-64 or on ARM (where it should run, not tried): its times are never the judge's, its program
size is not on ARM, and a program that reads a fine clock or makes a call the gate refuses passes it and dies at
the judge (AGENTS.md, "Commands").
