# The development runner (`bee/local/`)

**Never run a volunteer's candidate with it.** It is for the project's own proof candidates, written by the
project's builders, and for nothing a stranger sent.

Why: `run.sh` starts ONE container with `--privileged`, so that a Docker engine can run inside it and the judge can
mount its memory file systems, as it does on a hosted runner. A privileged container is not a wall: whatever runs
as root in it can reach the machine it runs on (its devices, its kernel's settings, the Docker volume it keeps). The
judge itself never runs a candidate as root, and runs each one in an unprivileged container nested inside, behind
its own walls; but those walls are then all that stands between a candidate and the machine, with the privileged
container's own counted for nothing. On a hosted runner the judge runs on a machine that is thrown away after the
run, and holds nothing; a machine this runner is used on is somebody's computer.

| File | What |
| --- | --- |
| `Dockerfile`, `entry.sh` | the runner's image: Ubuntu (the judge's own, by fingerprint), Docker's engine, Python; the engine starts at entry |
| `run.sh` | runs one command in it: this project on `/project` and a tree of the hive on `/hive`, both read-only, a scratch folder on `/scratch` if `BEE_SCRATCH` names one; `--network` for the image steps alone; two processors and two gigabytes, which bound every container nested in it |
| `assemble.sh` | inside it: a copy of the hive's tree in `/kind`, from which the judge runs (the hive's tree is mounted read-only) |
| `standin.py` | a stand-in text made from the hive's own files, for timings and plumbing only |
