# Design note

*The third pilot project of La Ruche, built on 21 November 2026 before anything of it was published, then brought
up to date: the corpus, the name and the licence were decided then (below, "Decisions"). Its judge, the hive's third
kind `relay`, was written with it and now lives in the hive (below, "In the hive"). Unless it says otherwise, every
figure here was measured in the development runner (`bee/local/`: the judge in one container limited to 2
processors and 2 GB).*

## In one page

- **What it is.** A compressor written from scratch in Rust (`src/`, 8 files, 390 lines): one order-2 context model,
  a binary arithmetic coder, and the empty places where the ladder goes (a mixer that averages, a refining step and
  a transform that do nothing). A model is a file of `src/models/`; the build finds it without a list.
- **What is measured.** `total_bytes` = the message's bytes + the bytes of the built program (`run`, one
  executable, linked statically). Exact. The judge measures the tip and the candidate in the same judgement;
  `sure_total_bytes` is 1 when the candidate's total is lower and its runs repeat.
- **Who judges.** Not the hive's common judge: its contract cannot express this (below). A third kind of judge,
  `relay`, written with this project in the shape of the second, on the hive's `container.py`, `reading.py` and
  `chrono.c`; it lives in Bee-ruche/hive (`tools/judge/relay.py`, `docs/relay-judge.md`).
- **What is proven** (all in containers, without network): the kind's own tests on a toy project (33, of which 15
  in containers, programs in C that can make any system call); on this project, on its corpus, the tip, an honest
  candidate accepted with its gain, and 15 others, hostile, each refused or shown to gain nothing (or, for `calls`, shown to reach nothing), for the reason
  expected.
- **The corpus.** 10,000,000 bytes of public-domain books (Shakespeare's works, three novels of Dickens), in the
  project's tree, pinned by SHA-256, made by a written rule from files whose own fingerprints are recorded. The
  baseline's total on it: 4,890,484 bytes, of which 27.5 % is the program as built.
- **What four independent reads found** (Sonnet agents given the code and the threats, not this note). The first:
  the phases' containers held the whole build image, a free dictionary. Fixed: phases run in an empty image. The
  second: the two phases still shared host folders (the launcher's, the program's), and nothing kept a program that
  can make system calls from leaving state in them or in the machine. Fixed: each phase has a memory file system of
  its own, and a gate of the kind's shuts every system call but reading and writing the phase's two descriptors,
  memory and ending. The third: a phase could still stop itself with a signal and wait, and a build's compiler
  could outlive the build's clock; both fixed (only SIGABRT is let through; the gate ends a build whole), and the
  gate shuts the time-stamp counter. The fourth: sentences of the kind's that said more than the code (a tick clock is
  left; what a tip that wavers does), now true, and the kind's failures on the tip's side named for the hive (below,
  "In the hive"). The fifth, of the
  kind as it went into the hive: a step of the tip's that fails in a way its bytes do not decide is now run once
  more before the failure is said to be on the tip's side. Smaller things, fixed or said (below).
- **What is open.** What only the project's own judgements on a hosted runner can show (times, what its kernel ends
  at a memory bound); a lighter frame around the program (1.3 MB of every total is the
  language's runtime and the C library today).

## Why not the common judge

The brief asked for the hive's common judge. Its plugin contract (`docs/common-judge.md`, `tools/judge/judge.py` at
d1e91fa) lacks four things this measure needs (the hive's `docs/relay-judge.md` says it too, "Why the common judge could not
judge it").

1. **A container's input that is the candidate's own output of another container.** There, every input is bytes
   the plugin wrote on the host before any run (`verification`, `repetition`), in one folder shared by the tip and
   the candidate; the decompression side works for miniz because the host owns zlib. Here the format is the
   candidate's: only the candidate's encoder can make the decoder's input.
2. **The plugin never sees the candidate's program.** What the decoder weighs could only be a number written by the
   judged process.
3. **An exact figure has no proof there, and costs 28 passes.** `sure_<metric>` is a sign test over at least six
   timed repetitions; a metric read once is never « beyond the noise ». Seven repetitions of two variants and two
   phases is 28 passes over the corpus inside an hour, where seven are enough.
4. **Data that is not in the tree** has no place: the image is a fixed Dockerfile plus package names.

So the judge was built as a third kind, beside the hive's two.

## The kind: `relay`

In Bee-ruche/hive: `tools/judge/relay.py` (the judge), `gate.c` (what shuts a phase off from the system's calls),
`test_relay.py` and `relay_toy/` (its tests and their toy project), `docs/relay-judge.md` (its specification). The
name says what is general in it: a program runs in two phases and the second is handed
the first's message and nothing else. A compressor is one such program; nothing in the file knows compression.

It has the second kind's command line and report (`--tip`, `--candidate`, `--task-file`, `--budget`,
`--tip-itself`, `--workdir`, `--json`, `--image-only`; codes 0, 1, 2; `{"kind", "code", "reason", "step", figures}`),
reads trees with the second kind's own functions (imported, not copied), and takes everything of the project from
a data manifest at the tip, `bee/relay/project.json`, each field held between a floor and a cap.

### One judgement

1. The manifest, the task's files, the budget, the tip. 2. The candidate's tree against the tip's; the program's
files; the data, held to its fingerprint. 3. The launcher's and the gate's build, and the tip's build. 4. The tip
encodes twice and decodes once (the tip against itself stops here): nothing of the candidate's has run yet. 5. The
candidate's build, and its two encodings. 6. The candidate decodes twice. 7. The figures. Seven passes over the
data, each phase in a container and on a memory file system of its own, behind the gate. A step of the tip's that
fails in a way its bytes do not decide (a signal, its processor time, its memory) is run once more before the
failure is said to be on the tip's side.

### Decisions, each with its reason

| Decision | Reason |
| --- | --- |
| **The count is the built executable's length**, not a packing of the source | The counted bytes are the very bytes both phases run, read by the host from the build's output and written again for the phases. There is no step between « counted » and « acts »: bytes in the source, in a second file, in a file's name, in a constant computed when the program is built, reach a phase only by being in that file. A first version counted the source through a fixed coder of the judge's own; it needed a coder kept identical for ever, priced comments, and the toy showed it could be cheaper than a weak program's own coding, so that moving data into the source « gained ». The executable has none of this: comments and names are free, and a byte costs a byte wherever it is put. It is also what the Hutter Prize counts. Its costs: figures are tied to one compiler, and the compiler's own choices move the size by hundreds of bytes: far under any task's floor. |
| **A phase runs in an empty image, and the program is linked statically** | The independent read's main finding. A phase used to run in the build's image: a gigabyte of compiler, libraries and licence texts that any program able to open a file could prime a model with, for nothing. Now the kind makes the image phases run in, and it holds one link and nothing else: no system, no shell, no library. What the program needs it brings, in the file that is counted. The cost is plain: the C library is now inside the count. This project's `run` went from 387,480 bytes (dynamic) to **1,346,472** (static), the same for the tip and a candidate. A frame written without the standard library would bring it to tens of kilobytes (open, below). |
| **Each phase has a memory file system of its own**, holding its own copies of the launcher, the gate, the program and its one input, unmounted when the phase ends | The second read's blocking finding. Before, the launcher's folder and the program's folder were the same host folders, mounted read-only into both phases: nothing could be written there, but a program able to make system calls could leave state in them that the next phase reads back (names looked up that do not exist, which pages are in memory). Now no folder of the host is ever given to two containers, as the common judge's `container.Execution` already does for its runs. (`Execution` itself is not reused: it builds its own launcher from `chrono.c` against the image's libraries and plans the harness's command; the kind needs a static launcher, the gate and its own command, so it copies the pattern, not the function.) |
| **A gate shuts every system call a phase does not need** (`gate.c`, the kind's own file; the hive's `chrono.c` is not changed) | A program is native code whatever its source promises: the language's rule and the compiler stand only as far as the compiler is sound, and it has known holes. So the gate, a program of the kind's, opens the phase's two files as descriptors 3 and 4, installs a seccomp filter that lets through reading and writing descriptors it holds, its own memory and signals, and ending, and only then starts the program (one execve, told by arguments that point into a page at a drawn address, which no longer exists once the program replaces the gate). Every other call (open, stat of any kind, `/proc`, `/sys`, `mincore`, the clock and sleeping as calls, other processes, threads, sockets, new descriptors) is refused before it is carried out. A filter installed by the launcher, rather than a profile given to Docker, keeps the hive's container code as it is and lets the two files be opened before the door shuts. The frames read descriptor 3 and write descriptor 4. |
| **Both phases are one program** (`run encode`, `run decode`) | One build, one file counted. Code only the encoder needs is counted too. The prize counts the decompressor alone; allowing a free encoder would need a second build that is given other files, and a rule on what may be in them. Left open. |
| **The tip is measured in the same judgement** | A gain is never the difference between two machines, two compilers or two days. |
| **The tip encodes twice too**, and a tip whose two messages differ is a failure to judge | A tip whose message moved from run to run would hand the next candidate a gain it did not make. Two runs do not prove a thousand would agree: they turn a moving figure into a loud failure that stops the judging and names the tip. Cost: one pass. |
| **A step of the tip's that fails in a way its bytes do not decide is run once more** | A kill for memory, processor time stretched past the tip's bound, a fault of the hardware may be the machine's, and the hive stops a run's judging on the judge's word that a failure is on the tip's side: a second run tells a blip from a fault. What the bytes decide (an exit status, a missing file, two messages that differ, a decoding that is not the data) is not run again, so that the two encodings keep meaning what they mean; a candidate's step never is. Cost: the worst case counts each of the tip's steps twice. |
| **The tip decodes once in every judgement** | A tip whose decoder stopped working (another compiler, another machine) is seen at the very next judgement, as a failure to judge, and no candidate is measured against it. Cost: one decoding pass (8 s for the baseline at 10^8 bytes in the development runner), and the bound's share of the worst case. |
| **A candidate encodes twice and decodes twice**, in four containers | « The same input gives the same message » is checked, not assumed. Two decodings, each compared with the data. |
| **The message crosses as bytes the host read** | The host reads `/out/message` through the hive's safe reader (no link followed, a size bound), keeps the bytes, and writes them on the decode phase's own new memory file system. Access and modification times there are fixed; change and birth times are the kernel's clock, and the gate lets no call read any time of a file. The encode phase's file system is unmounted and gone before the decode container exists. |
| **Only SIGABRT, and only to itself** | The third read: with any signal let through, a program stops itself (SIGSTOP), uses no processor, and the clock answers « could not say » for it, about two minutes and a half of machine time an attempt, for nothing. A C library and a language's runtime abort with SIGABRT, which is the one let through. |
| **The gate shuts the processor's time-stamp counter** (`prctl PR_SET_TSC`) before the program starts | Tried, not taken from memory (on a development machine's kernel, and in the kind's own tests on GitHub's hosted runner): the setting survives `execve`, and the counter and the fine clocks of the time the kernel maps into every process then end the program (SIGSEGV), since those read the counter; where a machine's clock is not read through it, they fall back to the call the filter refuses. **A tick clock is left**, and the gate cannot take it away: the coarse clocks of that mapped time read only the page the kernel updates at each tick (1 to 4 ms), as plain memory (tried: it reads and moves; a test pins it). So the time channel between phases is narrowed to the tick and the host's jitter, not closed. Whether some machines have other counters this setting does not cover is not established. Address randomisation could not be switched off: Docker's own filter refuses the call (`personality`, tried). `getrandom` stays: the drawn bytes and the addresses stay anyway, and a language's library uses it for its hash tables. |
| **A build is ended whole at its clock** (the gate's build mode) | The third read: the hive's launcher ends its own child alone, so a compiler made to run past the clock lived on, kept the launcher waiting until the container's own deadline, and the judge answered « could not say ». Now the gate holds every process the build leaves (subreaper), ends them all at the clock and reaps them, and the launcher reads all their processor time: past its bound, the candidate's (1); unused, the machine's doubt (2). |
| **Time is processor time, read by `chrono.c`**, for builds as for phases | The kernel's count for the process, printed by the hive's launcher under a token the program never sees; for a build, the shell's, which counts the compiler and every process it waited for. A clock would make a busy machine refuse an honest candidate: the first version bounded builds by the clock alone, and the second read found it. |
| **The clock is half as much again** (`bound + bound/2 + 5`) | It ends a program that never ends. Reached with the processor time **used**: the candidate's doing (1). Reached with it **unused**: a starved machine or a program that waits; nobody can say, so the judge does not (2). |
| **The tip's own program has a quarter more time** | A program accepted at the bound must not make every later judgement fail on a slightly slower day. |
| **Memory is the container's** | The kernel refuses the next page: no sampling, no noise. It counts what the program writes in its output folder too (a memory file system): said in the manifest's bound. |
| **One process**: `--pids-limit 2`, the launcher and the program | A thread or a child cannot start. |
| **The data: a file of the tip, or an address** | A toy keeps its data in its tree and its tests need no network. A real corpus is fetched once at the image step, held to two fingerprints (what is fetched; the data), kept as an image of one file in the machine's Docker, and read out again as bytes, held to its fingerprint, at each judgement. Never fetched while judging. |
| **A fingerprint of zeros is refused** | A project that has not chosen its data must not be judged on whatever answers at an address. As a workflow's forty zeros. |
| **http is taken as https is** | What is fetched is held to its fingerprint and the request carries nothing. Refusing http would rule out a corpus whose only public address is http. |
| **A candidate may change the `program` paths only** | The judge refuses a tree that differs from the tip's anywhere else, before anything runs. The build is given the tip's skeleton and the candidate's program files, written by the judge with fixed modes and times: a file brings its path and its bytes. |
| **The data is never under the skeleton or the program** (checked in the manifest) | No build is ever given the data. |
| **The gain is written rounded down** | A gain is never recorded larger than it is, by even a millionth. |
| **A candidate's tree is read no deeper than 32 folders** | Deeper is refused (1), before Python's own recursion would have turned it into « internal error » (2). |

### What is a refusal and what is « could not say »

| What happened | Code | Why |
| --- | --- | --- |
| the tree differs outside the program; the program's files are not plain, too many, too heavy; the tree holds more entries than the judge takes, or is nested deeper than 32 folders | 1 | the candidate brought it |
| the candidate's build fails or passes its processor time | 1 | built from the tip's skeleton, which built the tip just before, on the same machine |
| a phase exits with a status, is ended by a signal, is killed before the clock (memory), or its container is ended by the kernel with the launcher in it and Docker's record says it was for memory (`OOMKilled`) | 1 | the program's doing |
| a phase's processor time is past the bound (ended or stopped) | 1 | far from noise by the margin the project tells its workers (below) |
| a phase leaves no file, a link, a pipe, a folder, a file past its bound | 1 | read through the safe reader; never followed |
| two encodings differ; a decoding is not the data (length or any byte) | 1 | the measure itself |
| the manifest, the task's files, the budget, the tip's tree | 2 | the project's |
| the data: not fetched, another fingerprint, the image step not run | 2 | the project's or the machine's; **never held against a candidate** |
| the tip's build or phases fail (twice, where its bytes do not decide), pass the tip's own bound, or its two encodings differ | 2 | on the tip's side: the project's, or the machine's |
| Docker, its runtime, an image, the launcher's or the gate's build, the gate's own setup (status 126 at the tip's first phase), the launcher's line, a status 137 without Docker's memory record, a tree that cannot be read | 2 | the machine's |
| a build or a phase stopped by the clock with its processor time **unused** | 2 | the machine did not give it a processor. Behind the gate a phase cannot wait at all (no call that waits is let through); a build may wait on its own processes. Never a refusal: a failure to judge, which the hive answers by its own rule ([protocol.md](https://github.com/Bee-ruche/hive/blob/main/docs/protocol.md)) |

**How the time bound is set, and the margin.** The manifest gives 90 seconds of processor time to each phase and
120 to a build. On the corpus the tip takes 0.35 s to encode and 0.83 s to decode, an order-3 model more 0.8 s and
2.3 s; the ladder's heavy end (many models, a match model, a mixer that learns) takes tens of times the tip's time.
90 seconds leaves that room and keeps a judgement short. The worst case of a judgement, every container at its
clock and every step of the tip's run twice, is `70 + 3 × (185 + 15 + 10) + 6 × (140 + 10) + 4 × (140 + 10) + 60 =
2,260 s`, under the 3,480 s the hive gives a judgement of an hour (a bound may grow without turning every judgement into a 2). An honest heavy
candidate at 50 s a phase is judged in about six minutes. Processor time varies with the machine's
processor and its neighbours by a tenth or two (not measured on a hosted runner: an assumption to check at the
first runs). So the charter tells every task: **stay under six tenths of the bound** (54 s a phase), scaled from the
tip's own figures on the hive's machine (`encode_seconds`, `decode_seconds` on each accepted line) by the ratio
`bee/check` shows on the worker's machine. A candidate between 54 and 90 seconds was told; one refused there was not
surprised by noise.

**Memory:** 2,048 MB for the container; the charter says 1,600. The tip takes 47 MB, an order-3 model more 79 MB:
the room is for many hashed models. A hosted runner has 16 GB.

**A build** is bounded by its processor time (120 s, where the tip's takes about ten), all its processes together;
the clock stops it at 185 s, and a build stopped there with its processor time unused is the machine's doubt (2).

## The program

| File | What | Today |
| --- | --- | --- |
| `src/frame.rs` | the message's layout, the loop that codes one bit at a time | length (4 bytes), then the coder's bytes |
| `src/context.rs` | what every part is told: the last bytes, the bits of the current byte, the scale | |
| `src/models/order2.rs` | one model: 2^24 counts of 16 bits, chosen by the two bytes before and the bits so far | the only model |
| `src/mixer.rs` | one prediction from all the models' | their plain average |
| `src/refine.rs` | a last adjustment | none |
| `src/coder.rs` | a 32-bit binary arithmetic coder, 12 bits of probability | |
| `src/logistic.rs` | `stretch` and `squash`, as tables computed with the four operations | |
| `src/transform.rs` | a rewriting of the text before modelling, and back | none |

- **Rust, `core` and `alloc` alone, no unsafe code, no crate, one compiler** (rustc 1.75, as the pinned Ubuntu
  packages it: the hive's `rust` image, which its second pilot already uses). The build refuses another version.
- **From scratch.** Every line was written for this project from the published techniques (context mixing as
  Mahoney's papers describe it, the binary arithmetic coder). No line was copied from paq, lpaq, cmix or any other
  compressor. AGENTS.md makes it the rule for contributions.
- **A baseline that leaves the ladder.** One order-2 model is honest (it compresses) and far from the state of
  the art. The mixer, the refining step and the transform exist as files with their interface and do nothing, so
  that each direction is one file to change. Nothing in the baseline knows what the corpus is.
- **A model per file, no registry.** `bee/relay/rules.py` (the project's, in the build's container) lists
  `src/models/*.rs`, writes the crate's root with one module per file and a `Set` that calls each in the order of
  the names, and gives the frame `models::INPUTS`. Adding a model is adding a file: proved by `honest-order3`,
  whose candidate changes no existing file.
- **The rule on what the program may hold** (`rules.py`: file names, printable ASCII, thirteen refused words, `#`
  for four attributes only), with the root the build writes (`#![no_std]`, `#![forbid(unsafe_code)]`). The
  language then gives a file of the program nothing but computing and allocating. Words are refused in the raw
  text, comments included, so that nothing has to be parsed; macros are refused so that no word or attribute can
  be spelled out of sight. **It is a rule of good order, as strong as the compiler is sound**: safe Rust has known
  holes (the reviews name the lifetime one, rust-lang/rust issue 25860), through which a program would get the
  system calls a C program has. The judge does not rest on the rule: it takes every program for such native code
  and shuts the system calls themselves (the gate). What the rule buys is a program that is the same on every
  machine, and a worker told at once which line it may not write.
- **The frame around the program** (`bee/relay/main.rs`) is the only code that touches a file, and is the
  project's own: it reads descriptor 3 and writes descriptor 4, which the judge's gate opened before it started
  (or a shell, in `bee/check`). It is what brings the language's runtime and the C library into the count (1,346,472 bytes). A
  frame written without them (its own entry point, four system calls, an allocator) would weigh tens of kilobytes.
  It is the architect's to write, not a task: about 1.3 MB off every total, the tip's and the candidate's alike.

## Threats, how each is closed, and its proof

« toy » is a test of the hive's `tools/judge/test_relay.py` (programs in C, where nothing stands between a
program and the system); « project » is a candidate of `bee/proofs/`, judged on the corpus.

| # | Threat | Closed by | Proof |
| --- | --- | --- | --- |
| 1 | The decoder reads the corpus instead of the message: from the image, a mount, what the encoder left | Two walls, each enough alone against this. The gate: no call that names a path is carried out. The container: built on an empty image, on a memory file system made for that phase alone (its own copies of the launcher, the gate, the program and the one input), unmounted when the phase ends; the corpus is in no image a phase runs in and a build is never given it | toy `test_the_decoder_finds_the_data_nowhere_but_in_the_message` (11 places and the environment, each refused, behind the gate); `test_each_phase_has_a_file_system_of_its_own…` (the same 11 and the environment with the gate's filter LEFT OUT: still refused; a program right only if it finds no file of any system: right; seven phases, seven file systems, none left); `test_the_same_program_is_right_once_both_doors_are_opened_on_purpose`: with the filter left out AND one host folder given to every phase, the cheat is taken for right; with only one of the two doors open, it is refused. So what refuses it is the judge's setup, each wall alone. project `finder`, against a tip whose rule is loosened to let the standard library in: the decoder walks every folder it can read for a file starting with the data's first 64 bytes, and finds none: `0 bytes where the data holds 10000000` |
| 1' | …or gets it through the build | The build is given the skeleton and the program; the manifest refuses data placed under either | toy `BUILD_TAKES_THE_DATA` (`.incbin` of the tip's data path): the build fails; project `word-include` |
| 2 | Data kept outside what is counted: in the source, the binary, file names, a second output, the environment, the image | What is counted is the executable the phases run, and their container holds nothing else: there is no « outside ». A second output is never read and never handed on; the environment is the launcher's fixed one; a candidate changes nothing but the program's files | toy `test_bytes_moved_into_the_program_are_counted_there`: the data in the source, an empty message: the program grows by the data, no gain; in a second file: the same weight. project `embedded-prefix`: 65,536 bytes of the data held by the program: the message is 29,682 bytes shorter, **the program exactly 65,536 longer** (1,412,008 for 1,346,472), the total 35,854 higher, `sure_total_bytes` 0 |
| 3 | A lossy round trip, by one byte, or another length | Both decodings compared with the data on the host, byte for byte; no more than the data's length plus one is read | toy `test_what_is_given_back_is_the_data_byte_for_byte` (last byte, first byte, one fewer, one more, twice, nothing); project `lossy`, `shorter` |
| 4 | Past its time or its memory | Processor time from the kernel, through the hive's launcher; the container's memory bound; a clock that ends what never ends | toy `test_a_phase_has_its_time_its_memory_and_one_process` (never ends, eats a gigabyte, forges a measurement line on every descriptor it has); project `slow` (stopped after 50.0 s, bound 30), `hungry` (killed) |
| 5 | Output that differs from one run to the next | Two encodings in two containers must be the same bytes, the tip's as the candidate's | toy `test_a_figure_that_changes…` (bytes drawn, the clock, an address); a tip that draws: a failure to judge; project `changing` (an address the system draws, written in the message: 3,544,020 and 3,544,020 bytes, not the same) |
| 6 | Needs the network, or more than one thread | No network in any container; room for one process | toy: `NEEDS_A_THREAD`, `NEEDS_THE_NETWORK` refused; `WRONG_IF_IT_FORKS`, `WRONG_IF_IT_THREADS`, `WRONG_IF_IT_REACHES_THE_NETWORK` right, which they are only if they can do none of it |
| 7 | Changes the judge's own ground: the frame, the build, the manifest, the bounds, the data | The tree must be the tip's outside the program; the hive protects `bee/**` and `hive/**` before that | toy `test_a_candidate_is_the_tips_tree_but_for_the_program`; project `outside-the-program` |
| 8 | A link, a pipe, a folder or a flood where the message should be | The hive's safe reader: nothing followed, a size bound; a bounded memory file system to write in | toy `test_what_a_program_does_to_the_file_it_leaves` |
| 9 | Forges its own measurement | The launcher's line carries a token the program never saw; the program's output goes nowhere | toy `FORGES` |
| 12 | Waits on purpose, to burn the judge's time with « could not say » | Behind the gate no call waits and no signal stops (SIGABRT alone is let through); a build is ended whole at its clock and all its processor time counted | toy: a program that sends itself SIGSTOP is right behind the gate (the call refused) and is « the clock's » with the filter left out; a program that aborts ends by SIGABRT; a build whose child spins past the clock: refused (1); a build whose child sleeps: 2; the tip's own spinning build: 2, after a second run |
| 13 | Calls the kernel another way: x32 numbers, the 32-bit entry | The filter ends the process | toy `X32`, `ENTRY_32`: ended by signal 31 |
| 14 | Reads the clock without a call (time-stamp counter, the kernel's mapped time) | The gate shuts the counter before the program starts | toy `test_a_phase_has_a_tick_clock_and_no_finer_one`: the counter and `clock_gettime` end the program (signal 11), and the tick clock that is left is pinned; with the counter left readable on purpose, a message that follows the clock is caught by the two runs instead |
| 11 | Leaves state in the machine for the next phase: names looked up, which pages are in memory, a file's change or birth time, `/proc` and `/sys` figures, a socket, a memory file, another process | The gate refuses every such call before it is carried out; no folder is shared between phases | toy `test_a_phase_makes_no_call_that_reaches_the_machine`: a program that tries 23 paths in 6 ways each (open, stat, statx with birth and change times, access, readlink, chdir) and fstat, mincore, the clock and nanosleep as calls, getcwd, kill, uname, memfd, a socket, and is right only if none was carried out: right. Control: with the filter left out, the same program reaches the machine and gives wrong data (refused). A second execve ends the process (signal 31). A program that means to sleep an hour goes on at once behind the gate; without it, it waits and the clock stops it |
| 10 | Reaches a file, a clock or code that runs before the frame, through the language | The rule and the crate root; the compiler | project `word-unsafe`, `word-extern`, `word-include`, `word-macro`, `attribute`, `no-std-library` |

**Not closed, and said.**

- **A soundness hole of the compiler** (rust-lang/rust issue 25860 and its kin). The second read asked for it to be
  tried for real on the pinned compiler. **It was not**: building a working exploit chain (safe code that turns the
  hole into calls of the C library's own) is more than this project should write or keep. It is also not needed to
  show what it would give: it would give a program the reach of a C program, and the toy's C programs have that
  reach and make every call there is to make. What they met is what such a program would meet: a gate that
  refuses every call but reading, writing, memory and ending, in a container of its own with nothing in it.
- **Timing, narrowed, not closed.** Behind the gate a decoder has a tick clock and no finer one: it can read when it
  started to within a tick (1 to 4 ms) and the host's own jitter between containers, which an encoder can move by
  taking longer for some data. A few noisy bits a judgement, outside the count, of no use against data of millions
  of bytes. (The counter's shutting was tried on a development machine's kernel, and holds on GitHub's hosted runner in
  the kind's tests there; where the gate cannot shut it, it stops and the judge fails to judge.)
- **What the gate cannot see**: it is written for x86-64 (elsewhere the judge fails to judge), and it rests on the
  kernel's seccomp, on which every container runtime already rests.
- **Determinism is tested, not enforced.** The addresses the system draws for a program's memory, sixteen bytes the
  kernel draws for it, and `getrandom` stay, and a program may let them decide, on purpose, one time in N. Two
  encodings and two decodings catch most; a decoder that fails one time in three passes its own two decodings four
  times in nine. Once such a program is the tip, every judgement encodes it twice and decodes it once, and sooner
  or later one fails: the judgement answers 2, on the tip's side (« the tip's own two encodings… », « the tip's own
  program does not give the data back », « on the tip's side (not the candidate's) », `"fault": "tip"`). The tip's
  whole figure is measured before anything of the candidate's is built or run, so nothing of that candidate's
  brings it about within the judgement (the machine is shared with the run's earlier judgements, and a program
  accepted earlier in the run is the later ones' tip); and the hive takes that word as it takes a judge that fails
  on the tip itself: the run stops judging, the candidate is not charged, the
  run ends as a failure. The project stalls, loudly, until the architect reverts the commit that brought the
  program in, on the hive's branch; the next run judges against the tip before it.
- **A program fitted to the corpus.** Fixed and public: tuning to it is the measure's nature, and every byte of
  tuning that reaches the program is counted.
- **A build that waits** (a compiler that sleeps, never using its processor time) is answered « could not say » three
  times before its deal ends: three clocks of a build's. A phase cannot wait behind the gate.
- **A build's `/tmp`**: the image's 64 MB, for whatever a compiler writes there beside the build's folder (this
  project's build writes its scratch files in the build's folder, `output_mb`).
- **The development runner** (`bee/local/`) is a privileged container (it holds a Docker engine). It runs
  the judge, never a candidate: candidates run in the unprivileged containers nested in it. On a hosted runner the
  judge runs on the machine itself, as the other kinds do.

## What the independent reads found

### The first read

A Sonnet agent was given the judge's code, the hive's files it stands on, the project's build and the list of
threats, and told not to read this note, the documents or the tests first. It ran nothing. What it reported, and
what was done:

| Finding (its severity) | Done |
| --- | --- |
| The decode container's root is the whole build image: a gigabyte of files a program able to open one could use as a dictionary nobody counts (high for the toy's C; through a compiler hole for Rust) | **Fixed**: phases run in an empty image made by the kind; programs and the launcher are linked statically. New toy test: a program right only if it finds no file of any system |
| The Rust rule and `forbid(unsafe_code)` are no wall against the compiler's known soundness holes, and the rule's text claimed a pure function (high, through the finding above) | The claim is **rewritten** (`rules.py`, AGENTS.md, here): a rule of good order; the container is the wall. The hole itself was not tried |
| A program that sleeps is answered 2 for ever (medium) | **Kept** at first: refusing on « little processor time used » would refuse an honest candidate on a starved machine. **Closed** after the third read: behind the gate a phase cannot wait at all |
| A candidate's tree with too many entries was answered 2; a tree that cannot be read was « internal error » (medium) | **Fixed**: 1 for the first; a plain failure to judge for the second |
| The launcher's build, if it passed its time, was a refusal (medium) | **Fixed**: the launcher is the hive's: a failure to judge |
| A build is bounded by the clock only, and blamed on the candidate (medium) | Kept at first; **fixed** after the second read (below): a build is bounded by its processor time |
| A hidden 64 MB cap on the build, in the image's `/tmp` (medium, suspected) | **Fixed here**: the build writes its scratch files in its own bounded folder (`output_mb`, now 256) |
| File times let a decoder read a few bits of the encode phase's length; `/proc` shows the machine's figures (low to medium) | Access and modification times fixed; « fixed » was said of all times, which was not true (the second read). Both **closed** after it: the gate lets no call read a file's time or open `/proc` |
| The gain was rounded to the nearest millionth: a candidate ten bytes short could be rounded up to its goal (low) | **Fixed**: rounded down |
| A cancelled judgement left its mounts and folder; a network error class was not caught by name; the task-file check took `..`; a program's own words could span lines in a reason; the budget did not count the launcher's build (low) | **Fixed**, each |
| The refused words are refused as plain identifiers too (`env`, `include`, `asm`, `macro`) (low) | **Kept**: the build names the line; AGENTS.md lists the words |

One more, from the builders' own second look: a tip whose message size moved between runs would have handed the
next candidate a false gain. **Fixed**: the tip encodes twice, and two different messages stop the judging.

It examined and found sound, by reading: the exact comparison and the safe reading of outputs; that the counted
bytes are the bytes that run; that no writable path is shared by the two phases; the builds' isolation from the
data; the tree comparison; the manifest's checks; the launcher's token and descriptors; the order that makes a
broken machine fail on the tip first; the rule's regular expressions against attributes, renames, raw
identifiers and nested modules.

### The second read

A reviewing session re-ran the tests and proofs of the version the first read left, then had another Sonnet agent read the kind without this
note or the first read. Its blocking point, which the reviewer confirmed in the code, and the rest:

| Finding | Done |
| --- | --- |
| **Blocking.** The launcher's folder and each program's folder were the same host folders, mounted read-only into both phases. A phase able to make system calls (C; or Rust through a compiler hole) can leave state there that the next reads back: names looked up that do not exist, which pages are in memory. And nothing kept such a program from the machine's state (`/proc`, `/sys`, times) | **Fixed**, two walls, each tested alone. Each phase has a memory file system of its own with its own copies (the common judge's `Execution` pattern). A gate (`gate.c`, the kind's own file; `chrono.c` unchanged) opens the two files and shuts every system call but reading and writing them, memory and ending. New toy tests: every channel tried by a C program, refused; each wall alone holds; both doors opened on purpose, the cheat works |
| Try the compiler's hole for real | **Not done**, and said under "Not closed": a working exploit chain is more than this project should write or keep; the C programs already have the reach it would give |
| The documents said more than was true (« not even a time », « times fixed », « cannot open a file », « an empty container ») | **Corrected** in `relay-judge.md`, here, `rules.py`, AGENTS.md, `bee/README.md` |
| A candidate's build was refused by the clock alone | **Fixed**: processor time, the clock half as much again, a doubt is 2 |
| A tree nested about a thousand folders deep made Python's recursion fail: « internal error », 2 | **Fixed**: deeper than 32 folders is refused (1) |
| The tip never decodes in a judgement | **Fixed**: once in every judgement (cost: one decoding pass). Two encodings and not more: said under "Not closed" and in `relay-judge.md` |
| Memory: a host that kills the whole container leaves « could not say » for a hungry candidate | **Fixed** both ways: the gate marks the program as the kernel's first choice (`oom_score_adj` 1000); a container ended with 137 and no measurement is read as memory (1). What a hosted runner does: to be seen at the first runs there |
| Notes for "What the hive needs": the corpus fetched at every run, `rustc` not pinned, the length of a judgement | **Written** there |
| The development runner is privileged | **Said** in `bee/local/README.md`, `run.sh` and `bee/README.md`: never a volunteer's candidate |

It found sound: which paths a candidate may change, the manifest's checks, the data never in a build's input, the
counted bytes being the executed bytes, the tip's total measured each time, the safe reading of what containers
leave, the subprocess arguments, the registration patch's isolation from the two existing kinds.

### The third read

The reviewer had another Sonnet agent read the version the second read left, and read `gate.c` itself. It found the channel between the
phases closed and the gate sound in its structure (installed before any byte of the program, default deny, the x32
calls and the 32-bit entry ended, the jumps checked by hand). Two points blocked, both confirmed in the code; neither
let a candidate win, both let it burn the judge's time and both contradicted the documents.

| Finding | Done |
| --- | --- |
| **Blocking.** `tgkill` was held to its first argument alone: a program could send itself SIGSTOP and wait, using no processor, for a « could not say » charged to nobody | **Fixed**: SIGABRT to itself alone. Tested: a program that stops itself is right behind the gate and « the clock's » without the filter; a program that aborts still ends by SIGABRT. The texts say what is now true: a phase cannot wait, so a phase stopped with its processor time unused was starved by the machine (2) |
| **Blocking.** A build past its clock: the launcher ended the shell, the compiler lived on, the container's deadline answered « could not say » | **Fixed**: the gate's build mode ends the whole build at its clock and counts all its processor time. Tested with a toy build whose child spins past the clock (1), whose child sleeps (2), and a tip whose build spins (2) |
| A program can still be right one time and wrong the next, on purpose | (a) Address randomisation: **cannot** be switched off: Docker's filter refuses the call (tried). `getrandom` **kept**, with the reason. (b) The time-stamp counter: **shut**, tried on the real kernel: it survives `execve`, and the fine clocks of the mapped time fault with it (a tick clock is left: the fourth read). (c) « Tested, not enforced », what a wavering tip does once it is the tip and how the architect recovers: **written** (`relay-judge.md`, "When the tip is at fault") |
| The time channel | **Narrowed**, not closed, by (b): the fourth read found the vDSO's coarse clocks, which read no counter; tried, a tick clock is left (1 to 4 ms), and a test pins it |
| The data stays in the judge's memory while the decoder runs | **Fixed**: while a decoder runs the judge keeps the data's length and SHA-256 alone, and reads the data again for the next encodings |
| Status 137 without a line was blamed on the candidate | **Fixed**: only when Docker's own record says `OOMKilled`; otherwise the machine's (2) |
| The budget fitted with 20 s to spare | The bounds set from the corpus (90 s a phase) bring the worst case to 1,600 s of the 3,480 |
| `close` of descriptors 5 to 1023 only | **Fixed**: `close_range`, the loop kept as a fallback |
| x86-64 only: what a volunteer on ARM meets | **Said** in AGENTS.md and `bee/README.md`: `bee/check` runs without the gate and should run on ARM (not tried); its program size and times are not the judge's |
| « each phase has a file system of its own » checked what was true by construction | **Fixed**: every host folder a phase is given must lie inside that phase's own root |
| The filter's tests did not try a stopping signal, an x32 call, the 32-bit entry, a filter that fails to install | **Added**, each |
| 137 classified in a unit test only | Still a unit test (with Docker's record mocked both ways): a whole-container memory kill could not be made to happen in the development runner, where the program is ended alone |
| Which project proof fails if the Rust frame does not run under the filter | None of the sixteen did: the per-phase file systems alone already leave `finder` nothing to find. **Added** `calls`, against the loosened tip: a Rust program with the standard library that tries `/proc`, its own program, the root folder and its working folder, right only if every call was refused |

### The fourth read

The reviewer had another model read the version the third read left, and read it too. B1 and B2 closed, nothing loosened, the corpus as
a file of the tip sound (rebuilt by the reviewer from the four fetched files: the same fingerprint). Left:
sentences that said more than the code, and one test.

| Finding | Done |
| --- | --- |
| « Behind the gate a program has no clock » was not true: the vDSO's coarse clocks read no counter | **Tried**: behind the gate `CLOCK_MONOTONIC_COARSE` reads and moves across a busy loop. Written as it is (a tick clock is left, known, not closed) in `gate.c`, `relay-judge.md` and here; the test is now `test_a_phase_has_a_tick_clock_and_no_finer_one`, with a program that pins the tick clock. The time channel is « narrowed to the tick and the host's jitter » |
| The kind's document said that no candidate pays for a tip that wavers, which was more than the code did | **Made true**: the kind measures the tip's whole figure before anything of the candidate's is built or run, and names a failure on the tip's side (« on the tip's side (not the candidate's) », `"fault": "tip"`), and the hive takes that word: for this kind, the run's judging stops and no candidate is charged. Tested on both sides, the kind's and the hive's |
| The data out of the judge's memory protects little: the trees hold it, in the host's file cache | **Said**, one sentence |
| The test's second half (a stray folder) could not fail | **Made able to fail**: it now reads the mounts as Docker is given them, finds no stray folder in an ordinary run, and finds the stray one, seven times, when one is added on purpose |
| A waiting build is a 2 by design, on a condition | **Said**: if a project's build script ever runs anything of the program's at build time, a waiting build becomes a candidate's free retry |
| ARM, and what `bee/check` does not meet | **Said** in AGENTS.md and `bee/README.md`: `bee/check` never goes through the gate, on any machine |

### The fifth read

The kind's pull request to the hive (its three commits: the container's memory record, the kind, the hive's rule)
was read by another model. Nothing blocked: no way was found for a candidate to make its own judgement answer 2
with the field, and the two other kinds are unchanged. Its other points, done in the kind with that pull request:

| Finding | Done |
| --- | --- |
| No second look before a tip is said to be at fault: a one-off failure of the machine during a tip's phase stopped the run's judging at once | **Fixed**: a step of the tip's that fails in a way its bytes do not decide (a signal, its processor time, its container ended for memory, a build stopped at its clock) is run once more; if the second run passes the judgement goes on, and the log says the first failed. What its bytes decide is not run again. The worst case counts it. Tested both ways, with Docker's launch played |
| « The tip's own doing » was said of failures that may be the machine's | **Rewritten**: « on the tip's side », taken as the tip's and not as proof that its code is wrong; what is shared beyond one judgement (the machine, a tip accepted earlier in the run); whether Docker's memory record is set by a shortage of the whole host is not established |
| Tests with little margin on a slow shared runner | **Widened** without loosening what they prove: the tick-clock program reads until the clock moves, not for a fixed count; the build-clock test times the build's own container, not the whole judgement |
| Two costs unsaid | **Said** in `relay-judge.md`: every judgement measures the tip's figure before the candidate's build; a tip at fault costs one measured tip in every run until it is mended |

## What was measured

On the corpus (`bee/corpus/data`, 10,000,000 bytes, SHA-256 `9ff1b7fe…6868d24`), by the judge, in the
development runner. Processor times are `chrono.c`'s.

| | The tip | `honest-order3` |
| --- | --- | --- |
| message | 3,544,012 (2.84 bits a character) | 2,987,868 |
| program, as built | 1,346,472 | 1,346,472 |
| **total** | **4,890,484** | **4,334,340** (gain 11.37 %, proof 1) |
| encode / decode (processor seconds) | 0.35 / 0.83 | 0.76 / 2.32 |
| memory, encode / decode | 47 MB / 47 MB | 79 MB / 79 MB |

The 17 proofs on the corpus all gave what was expected (`embedded-prefix`: 65,536 bytes of the corpus held by the
program; the message 29,682 bytes shorter, the program exactly 65,536 longer, the total 35,854 higher).

At ten times the size, on a stand-in (`bee/local/standin.py`: words of the hive's own files strung again by seeded
chance, 100,000,000 bytes, SHA-256 `fe1c3a91…3c82409e`), the same programs take 3.70 / 8.41 s and 7.96 / 25.33 s
and 166 and 190 MB: what a corpus of that size would cost, and that the plumbing holds at full size inside the
runner's 2 GB. No size measured on the stand-in says anything about the corpus.

- Decoding is slower than encoding (the processor cannot look ahead at bits it has not decoded): the decode bound
  is the one a heavy program meets first.
- The program is 27.5 % of the baseline's total, and nearly all of it is the language's runtime and the C library,
  linked statically into the project's frame: the charter makes the program's own size a direction (the frame,
  the architect's own; the compressor's code, a task's at the margin).
- `bee/check`, in the sandbox's image, gives the judge's own figures on the same bytes.
- **On a hosted runner** (not measured): one core of a shared machine is plausibly as fast as this one or up to
  twice slower. The baseline's judgement would be seven passes of one to five seconds, three builds of about ten
  seconds and the copies: about a minute. To measure at the first run and write in the charter.
- The kind's test file: 33 tests, about 390 seconds in the runner, 15 of them in containers.

## In the hive

**The kind's name:** `relay`. Two phases, the second with the first's message alone; nothing in it is a
compressor's.

**What came into the hive with it** (Bee-ruche/hive at `ddaeb008044eb431d1572ea4d73848407069b5b2`, the commit this project's workflows name): the kind
(`tools/judge/relay.py`, `gate.c`, `test_relay.py`, `relay_toy/`, `docs/relay-judge.md`) and its registration;
`container.Docker.launch` handing back Docker's own record of a memory kill (`oom_killed`); and the hive's rule for a
judge's word that a failure is on the tip's side (`"fault": "tip"`): nothing more is judged in the run, the
candidate it was on is not charged, the run ends as a failure, the log names the task. The hive's own tests run the
kind in a job of their own.

`TIP_KINDS` stays `("reference",)`: this kind's tasks ask a gain, never « correct ».

**What the image step requires of the network.** As the other kinds: the base image and Ubuntu's packages. And,
for a project whose manifest gives an address: **that address, once per machine**, at `hive.py image`, before any
candidate's code is there. A hosted runner is a new machine at every run and `hive.py image` runs at every run
that is not paused: so **the data is fetched at every run** (the image is rebuilt at every run too, as for the
other kinds). For a large file from a third party's server, that is a load and a dependency (this project keeps its
corpus in its tree for that reason, below). The hive could spare it by making the image only when the plan holds a
candidate: a change of `cmd_image`, not asked here; or a cache of the judging job keyed by the data's fingerprint
(the workflow's `actions/cache`), which the hive's run does not use today. The empty image phases run in is made
without network.

**When the data cannot be fetched, or is not the manifest's:** the judge fails to judge (2) and never refuses a
candidate. At the image step (the address does not answer, what was fetched has another size or fingerprint, the
archive holds no such member) `hive.py image` fails and nothing is judged in that run. While judging (the image
step did not run, the kept bytes are not the manifest's) every judgement answers 2, the tip against itself
included, so the hive's rule holds: the machine cannot judge, the run's judging stops, no deal is charged.
`relay-judge.md`, "The data", lists each case and the judge's words. Tested without network in `TestTheData`.

**What the kind asks of a project** that the other kinds do not: a build that leaves **one statically linked
file**, and a program that reads its input on descriptor 3 and writes its output on descriptor 4 (the gate opens
them; the program can open nothing).

**What the kind asks of the judge's machine**: as the common judge, the right to mount memory file systems (root,
or `sudo -n`, which a hosted runner gives), one per phase; Docker with its default system-call filter; an x86-64
processor (the gate is written for it) whose time-stamp counter a process can shut for itself (`prctl PR_SET_TSC`,
x86's own; where it fails the gate stops and the judge fails to judge).


**The compiler is not pinned.** The image installs `rustc` as Ubuntu's archive serves it on the day (`apt-get
install rustc lld`, the hive's `rust` image): a later security update of the same release can change the program's
size by some bytes. Within one judgement that changes nothing (the tip and the candidate are built by the same
compiler and the tip's total is measured again), so a gain stays a gain; across judgements the figures in the
ledger may move by those bytes. The project's build refuses another release than 1.75.

**What a run costs.** A candidate's judgement: the launcher's and the gate's build, two builds and seven passes
over the data. Worst case, every container at its clock and every step of the tip's run twice: 2,260 seconds with
this project's manifest (a kind that
lets a project set bounds near the hour lets a judgement take that long: four of them fill a run's four hours of
judging, `limits.judging_seconds`, and the next candidates wait for the next run). Expected for the baseline on a
hosted runner: about a minute (above; not measured). The tip against itself: one build, three passes (two and six
at most). Every judgement measures the tip's figure before the candidate's build, so a candidate that does not build
costs that figure all the same.
The image step: the image (about a minute of packages), and, for a project whose data is an address, its download.
This project's data is a file of its tree: its image step fetches no data, and no judgement reads anything but the
tree the base wrote out.

**Proved before it went in.** On an export of the hive at that commit (never a clone's own tree): the hive's own
suites, without network, in its test image: `test_hive` (528 tests), `test_schedule` (6), `test_page` (117),
`test_artifact` (31) pass; `test_judge` (136) and `test_reference` (31) end with the same 15 errors and 2 failures
as at d1e91fa, the test image's own (it has no `sudo`, and the copy it was given kept no file modes), and their
container tests are skipped there. In the development runner: `test_relay.py` whole (33 tests, none skipped,
through `hive.judge_of` and `hive.judge_image` too); the seventeen proofs with lowered bounds, and the tip and the
honest candidate with the manifest's own (the figures above). `bee/proofs/loads.py` loads this project's settings as
they are. On GitHub's hosted runner, at the pull request's first push, the hive's jobs « The judge's third kind »
and « The judges » passed: the gate, the time-stamp counter and the 32-bit entry held there.

## The corpus

**What happened.** The project was designed for « enwik8 » (the first 10^8 bytes of a Wikipedia dump, as the Large
Text Compression Benchmark publishes it). On 21 November 2026 the benchmark's site was refused by the
security software of the machine the project was built on (flagged as « riskware »), from the host and from
containers. That refusal was taken as final: nothing was tried to get round it and no byte of enwik8 was fetched from
anywhere. A volunteer whose machine runs such software could not have built a sandbox from that address either.
Three alternatives were written down (enwik8 from the benchmark, were the site allowed; the same bytes re-hosted by
the project, with Wikipedia's share-alike duties; another fixed corpus from a host that is not flagged), and the
third was chosen: public-domain books, smaller than enwik8, kept in the project's own tree.

**What it is.** The first 10,000,000 bytes of Shakespeare's complete works, then David Copperfield, Bleak House and
Our Mutual Friend (its first 617,236 bytes), each as its plain UTF-8 text with the distributor's header, footer and
licence taken away, one line feed between two books. SHA-256 `9ff1b7fed5395208b23d20ce67fc5b0b4cf93b384b4dcba6c4c793b8f6868d24`.
The README lists each book, its Project Gutenberg number, the fetched file's length and SHA-256.

**Where from.** Project Gutenberg's transcriptions, fetched once, on 21 November 2026, from the Project Gutenberg
Literary Archive Foundation's own server (`https://gutenberg.pglaf.org`; gutenberg.org was down for maintenance,
503; that host and no other was used): one folder listing and four files, with a generic client
name, into a new empty folder, read as data. Each file was held to what it says it is (its start line names the
book's number, its first lines the title and the author).

**How.** `bee/corpus/make.py` holds each fetched file to its recorded length and SHA-256, applies the rule its text
writes down (the lines between the distributor's start and end lines; a transcription credit and a transcriber's
note dropped; blank lines at the ends; a stop if the distributor is still named), joins, cuts, and holds the result
to the corpus's SHA-256. The rule was mended once, by hand, when Bleak House's transcriber's note was found to name
the distributor: the rule, not the text. Afterwards no line of the corpus holds « Gutenberg », « www. », « http »
or « pglaf »; the other words a distributor uses (« ebook », « licence », « produced by », « donation ») occur in
the works themselves (a note-book, Fortinbras's licence, « produced by D. »), and were read one by one.

**Why in the tree.** Through the kind's « a file of the tip » route, the one its toy uses: no judgement, image step
or sandbox build fetches anything, and nothing depends on a third party's server being up. Nothing breaks for a
10 MB file: a build is never given it (the manifest refuses data under the skeleton or the program); a judgement
reads it once and compares the candidate's tree with the tip's byte for byte (10 MB, under a second); the hive's
limits on a candidate (`limits.file_bytes`) bind what a candidate changes, and no candidate may change `bee/`; a
worker's `hive start` hashes it with the rest of the tip. `.gitattributes` keeps git from converting its line ends
in a clone (`-text`).

**What is lost and gained.** No figure here can be placed beside published enwik8 figures; the aim is to try the
hive, not to be placed. Gained: no licence duty (the texts are in the public domain), no flagged host, a text no
published compressor was tuned on, judgements three to ten times shorter than on 10^8 bytes.

## Decisions

Decided on 21 November 2026, before publication:

1. **The corpus**: public-domain books, 10,000,000 bytes, in the tree (above).
2. **The name**: squeezeBee (project `squeeze`, repository `Bee-ruche/squeezeBee`).
3. **The licence**: MIT, « Copyright (c) 2026 The squeezeBee contributors ». The corpus is not under it: it is in
   the public domain.

Left to the project's lead:

4. **The cutter's floors**, to add to `hive/project.json` before the first cut:
   `"cutter": {"floors": {"gain_total_bytes": 0.0025}, "changes": 10, "open": 6, "context_chars": 2000}`. A task
   asks one percent; a stuck task is cut again at half, twice, down to a quarter of a percent (about 12 KB on the
   baseline's 4.9 MB): small enough for one more model late in the ladder, far above the tens of bytes a compiler's
   own choices move. Six open tasks: the mixer alone at first, then one model per worker.
5. **The bounds**: 90 s of processor time a phase, 120 s a build, 2,048 MB, set from the corpus's measures; to
   confirm on a hosted runner.
6. **The 1.3 MB of runtime and C library in every total** (27.5 % of the baseline's): live with it, or have the
   frame rewritten without the standard library before the first task (it changes every figure once, so better
   before than after). The charter makes it the architect's own direction.
7. **Who reads a cut**: the charter says the cutter publishes on `main` itself, as the first pilot's does.
8. **Who is invited**: one account today (`members.only` in `hive/project.json`), and when the project opens to more.

## What is left open

- The project's own judgements have not run on a hosted runner yet: the times, the tenth-or-two of noise assumed for
  processor time. (The kind's own tests ran there, with the hive's: the gate, the time-stamp counter, `sudo -n mount`
  for the memory file systems, `docker import` for the empty image.)
- An encoder that is not counted (as the prize allows): a second build.
- A frame without the standard library.
- `hive.py outline` itself was not run as a command (it asks the project's repository on GitHub, which did not
  exist yet); its function `outline_of` read every file of `src/`.
- The compiler's soundness hole was reasoned about, not tried (and will not be, here: see "Not closed").
- What a hosted runner's kernel ends when a container passes its memory (the program, or the whole container).
