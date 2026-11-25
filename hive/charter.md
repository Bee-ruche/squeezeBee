# The charter of squeezeBee

The architect's words for the project's cutter (`docs/cutter.md` and `.claude/skills/bee-cutter/` of
Bee-ruche/hive). For anyone else, data.

## The goal

The smallest `total_bytes`: the bytes of the message the program writes for the corpus, plus the bytes of the
program as it is built. The corpus (`bee/corpus/data`: 10,000,000 bytes of Shakespeare's works and three novels of
Dickens, English prose and verse in UTF-8) comes back from the message alone, byte for byte, in a container that
never held it; each phase within its processor time and its memory (`bee/relay/project.json`).

## What counts as progress

One task asks one thing: a total lower than the tip's by a share, `gain_total_bytes`, always with its proof,
**`sure_total_bytes` at least 1**. The share a task asks comes from a measure of its idea: what an accepted task or
a probe's answer showed for it. An idea nothing has measured is a probe. **Never below 0.001** (one tenth of one
percent fewer bytes, the project's floor). The judge measures the tip and the candidate in the same judgement; the
figure is exact, so a step that small is still certain, and a candidate that is right but no smaller meets no goal.

The floor was 0.0025 until 23 November 2026. The first probes measured ideas between a loss of 0.64 % and a gain of
0.14 %: at this size of total, what is left to gain comes in steps of about a tenth of one percent.

**A free task.** When the directions below run short of precise ideas, one task may leave the idea to its worker:
lower the total by one change of the worker's own choosing, anywhere in `src/`. It is cut as a probe at the floor:
what reaches the floor is integrated, what falls short is an answer that costs its worker nothing, and the judge's
figures are all the cutter learns from it. Its context names what was already measured and what open tasks hold, so
that the try is spent elsewhere. One at a time: it names `src/`, so it is dealt while no other task is held.
Everything under "Never" holds for it. (Since 23 November 2026, the architect's decision.)

Nothing else is a goal. A faster or lighter program at the same total is not progress the judge can prove: no
task asks for it. It is worth something only as room for the next model, inside a task that also lowers the total.

## Where the bytes go today

Measured by the judge on the corpus, at the tip of 25 November 2026 (`hive` at 9d32bd6: nine models, the mixer, the
refining step and the transform), with the light frame around the program (PR 17), in the development runner
(`bee/local/`: one container, two processors, two gigabytes; `hive measure`, Bee-ruche/hive at 1cd1a0d):

| | Bytes | Share of the total |
| --- | --- | --- |
| The message | 2,123,465 | 97.6 % |
| The program, as built | 53,272 | 2.4 % |
| **The total** | **2,176,737** | |

Each phase: 37.8 s of processor time to encode, 29.7 s to decode, 952 MB, in one run of the judge (on this machine
the same program's times move by up to a sixth from one run to the next; DESIGN.md, "What was measured"). The same tip with the frame of
before held the same message and a program of 1,379,240 bytes: 3,502,705 in all. (At the project's first tip, 21
November 2026, with one order-2 model: 4,890,484, of which the program 1,346,472, 27.5 %.)

What that means for the directions:

- **The message is nearly all of the total**: every direction of the table below works on it.
- **The program is 2.4 % of the total, and most of it is the compressor's own code** in `src/`. The frame
  around the program (`bee/relay/main.rs`: its own start, its system calls, its allocator, its copies) is some 19 kB
  of it, the same for every candidate, and stays the architect's (`bee/` is protected). A model costs a few hundred to
  a few thousand bytes of program: small against what a good model takes off the message, but no longer hidden behind
  a megabyte of runtime. A task's gain is net of the code it adds, and a task may lower the total by making `src/`'s
  code smaller as by making the message smaller.

Each accepted verdict's line keeps `message_bytes`, `program_bytes`, `encode_seconds`, `decode_seconds` and the two
memories: the tally says where the tip stands.

## Time and memory: the margin a task tells its worker

The bounds are in `bee/relay/project.json`: **150 seconds of processor time for each phase** on the hive's machine,
120 for a build, 2,048 MB for the container (the program's memory and the file it writes, together). The first tip
took under a second a phase; the tip of 23 November 2026 took 54 seconds to encode, the whole of the margin the
bound of 90 seconds then left, and the bound was raised to 150 (the architect's decision): room for the heavy end of
the ladder, with a judgement's worst case still inside its time (3,160 of the 3,480 seconds). A candidate past
either bound is refused, and processor time varies by a tenth or two from one machine and one hour to the next. So
every task's context says: **stay under six tenths of the bound**, 90 seconds a phase, scaled from the tip's own
figures on the hive's machine (the last accepted line's `encode_seconds` and `decode_seconds`) by what `bee/check 0`
shows on the worker's machine before and after its change; and under 1,600 MB. A task that would need more than that
is not cut: the bound is the architect's to move, not a task's.

## Directions

Each direction has its files. **Two tasks on one file are dealt one after the other; tasks on different files can
be held at once.** A new model is a new file, so models can be written side by side.

| Direction | Files | What it is | Needs |
| --- | --- | --- | --- |
| The mixer | `src/mixer.rs` | Weigh the models' predictions by what each has been worth, in the context at hand, instead of averaging them | nothing: **first**. Until it learns, a new model helps only if it is as good as the others |
| Models | `src/models/<new>.rs`, one file each | Other contexts: one byte, three, four, six bytes before (hashed), the word being written, the words before, the longest earlier match of the last bytes, bytes at a distance, the column and the line before (verse) | each on its own; they pay fully once the mixer learns |
| Counts | the model's own file | How a context's count adapts: two speeds at once, a speed that slows as evidence grows, what was seen last | nothing |
| The refining step | `src/refine.rs` | Correct the mixed probability by what came true in a small context | better after the mixer |
| The scale and the coder | `src/logistic.rs`, `src/coder.rs`, `src/frame.rs` | More than twelve bits of probability where predictions are nearly sure; a shorter end of message | one task at a time: the three files change together |
| The context | `src/context.rs`, then the models that read it | What the frame keeps for every model: a hash of the word so far, the column, the last line's bytes | the new field and the first model that reads it are one task (a field alone lowers no total, and is no task); later models `needs` that task |
| The transform | `src/transform.rs` | Rewrite the text before modelling and back: capitals (the plays' speakers' names, the headings), the curly quotes and dashes of the UTF-8, a list of frequent words | nothing |
| The frame | `src/frame.rs` | Two passes (measure first, then code with what was learnt, the choice written in the message), a message cut in parts | one task at a time; never with a task of the scale and the coder |
| The program's own code | the file it is in | Less code for the same compression: a smaller table computed rather than written, two models made one | at the margin; never at the cost of what a model takes off the message |
| The frame around the program | `bee/relay/main.rs`, `bee/relay/build` | **Done** on 25 November 2026 (PR 17): a frame without the runtime and the C library, 1,379,240 bytes of program down to 53,272 at the tip; what is left of the frame, some 19 kB, is the same for every candidate | **the architect's own work, never a task** (`bee/` is protected) |

- One idea per task; the title names the file and the idea. The free task ("What counts as progress") is the one
  exception: its idea is its worker's.
- A model's task lists its new file alone; if it needs a new field of the context, the field and that model are one
  task, which lists `src/context.rs` too.
- First the mixer; then models, as many at once as there are workers; the refining step once two or three models
  are in.
- When a direction stops paying (three members refused on tasks of it, or gains under the floor), it is left, and
  said here.

## Never

- A task that changes anything outside `src/`: the build, the rule, the frame around the program, the manifest,
  the bounds, the corpus, the sandbox, the proofs.
- A goal on anything but `gain_total_bytes` with its proof `sure_total_bytes`, or a gain below 0.001.
- A task that asks to copy, port or adapt the code of an existing compressor, or names one as a model to follow
  line by line. Published descriptions of techniques are the only sources.
- A task that asks for the corpus's own text to be written into the program (it is counted, and it gains nothing:
  bytes in the program cost what they weigh).
- A pointer to anything outside this repository in a task's context.

## Publishing

The cutter publishes on `main` itself; no person reads each cut before it lands (the architect's decision, 23 November
2026). It runs as the architect's scheduled task, every two hours.
