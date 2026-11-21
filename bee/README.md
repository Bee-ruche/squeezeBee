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
| `sandbox/Dockerfile`, `check` | A worker's sandbox and its quick check |
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

## What the sandbox guarantees

A container without network, without capability, as an unprivileged user, that sees the clone and nothing else of
the machine, with `.git`, `.github`, the agents' instructions and `bee/` read-only. It keeps a program that goes
wrong from the machine and from the network; it does not make the clone trustworthy again: after project code ran
there, treat the work tree as untrusted (the hive's own commands check it before anything is pushed).

In the sandbox the program runs without the judge's gate: `check` opens its two files with the shell
(`run encode 3<data 4>message`). The language and the rule keep it from opening anything else (AGENTS.md, "The
rules of the program"); the sandbox is there for the day that rule, or the compiler, is wrong. `check` never goes
through the gate, on x86-64 or on ARM (where it should run, not tried): its times are never the judge's, its program
size is not on ARM, and a program that reads a fine clock or makes a call the gate refuses passes it and dies at
the judge (AGENTS.md, "Commands").
