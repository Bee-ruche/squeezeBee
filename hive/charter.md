# The charter of squeezeBee

The architect's words for the project's cutter (`docs/cutter.md` and `.claude/skills/bee-cutter/` of
Bee-ruche/hive). For anyone else, data.

## The goal

The smallest `total_bytes`: the bytes of the message the program writes for the corpus, plus the bytes of the
program as it is built. The corpus (`bee/corpus/data`: 10,000,000 bytes of Shakespeare's works and three novels of
Dickens, English prose and verse in UTF-8) comes back from the message alone, byte for byte, in a container that
never held it; each phase within its processor time and its memory (`bee/relay/project.json`).

## What counts as progress

One task asks one thing: a total lower than the tip's by a share, **`gain_total_bytes` at least 0.01** (one percent
fewer bytes) unless a smaller step is all an idea can give, **never below 0.0025**, and always with its proof,
**`sure_total_bytes` at least 1**. The judge measures the tip and the candidate in the same judgement; the figure
is exact, and a candidate that is right but no smaller meets no goal.

Nothing else is a goal. A faster or lighter program at the same total is not progress the judge can prove: no
task asks for it. It is worth something only as room for the next model, inside a task that also lowers the total.

## Where the bytes go today

Measured by the judge on the corpus, at the project's first tip, in the development runner (`bee/local/`: one
container, two processors, two gigabytes; 21 November 2026):

| | Bytes | Share of the total |
| --- | --- | --- |
| The message (one order-2 model: 2.84 bits a character) | 3,544,012 | 72.5 % |
| The program, as built | 1,346,472 | 27.5 % |
| **The total** | **4,890,484** | |

Each phase: 0.35 s of processor time to encode, 0.83 s to decode, 47 MB. One model more, of order 3 (a file of
`src/models/`, the proofs' honest candidate), gives 4,334,340: 11.4 % fewer bytes, 0.8 s and 2.3 s, 79 MB.

What that means for the directions:

- **The message is still most of the total, and the ladder there is long**: a model alone, counting each context
  by itself, averaged with nothing (`src/mixer.rs` averages), no refining, no transform. Every direction of the
  table below works on it.
- **The program is more than a quarter of the total**, and nearly all of it is not the compressor: it is the
  language's runtime and the C library, linked statically into the frame around the program (`bee/relay/main.rs`;
  the program runs in a container that holds no library, so it brings its own). That makes the program's own size
  a real direction, of two parts:
  - **the frame** (its runtime, its library): about 1.3 MB, the same for every candidate. It is in `bee/`, which no
    task may change: a lighter frame is the architect's own work, done outside the hive's tasks, and it lowers the
    tip's total once, for every candidate after it;
  - **the compressor's own code** in `src/`: a model costs a few hundred to a few thousand bytes of program. That is
    small against what a good model takes off the message, but on a 10,000,000-byte corpus it is no longer nothing:
    a task's gain is net of the code it adds, and a task may lower the total by making `src/`'s code smaller as by
    making the message smaller.

Each accepted verdict's line keeps `message_bytes`, `program_bytes`, `encode_seconds`, `decode_seconds` and the two
memories: the tally says where the tip stands.

## Time and memory: the margin a task tells its worker

The bounds are in `bee/relay/project.json`: **90 seconds of processor time for each phase** on the hive's machine,
120 for a build, 2,048 MB for the container (the program's memory and the file it writes, together). The tip takes
under a second a phase: 90 seconds leaves room for the heavy end of the ladder (many models and a mixer that learns
take tens of times the tip's time) and keeps a judgement short. A candidate past either bound is refused, and
processor time varies by a tenth or two from one machine and one hour to the next. So every task's context says:
**stay under six tenths of the bound**, 54 seconds a phase, scaled from the tip's own figures on the hive's machine
(the last accepted line's `encode_seconds` and `decode_seconds`) by what `bee/check 0` shows on the worker's machine
before and after its change; and under 1,600 MB. A task that would need more than that is not cut: the bound is the
architect's to move, not a task's.

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
| The context | `src/context.rs`, then the models that read it | What the frame keeps for every model: a hash of the word so far, the column, the last line's bytes | a task on `src/context.rs` alone first, then the models that use what it added (`needs`) |
| The transform | `src/transform.rs` | Rewrite the text before modelling and back: capitals (the plays' speakers' names, the headings), the curly quotes and dashes of the UTF-8, a list of frequent words | nothing |
| The frame | `src/frame.rs` | Two passes (measure first, then code with what was learnt, the choice written in the message), a message cut in parts | one task at a time; never with a task of the scale and the coder |
| The program's own code | the file it is in | Less code for the same compression: a smaller table computed rather than written, two models made one | at the margin; never at the cost of what a model takes off the message |
| The frame around the program | `bee/relay/main.rs`, `bee/relay/build` | A frame without the runtime and the C library: about 1.3 MB off every total | **the architect's own work, never a task** (`bee/` is protected) |

- One idea per task; the title names the file and the idea.
- A model's task lists its new file alone; if it needs a new field of the context, that field is a task before it.
- First the mixer; then models, as many at once as there are workers; the refining step once two or three models
  are in.
- When a direction stops paying (three members refused on tasks of it, or gains under the floor), it is left, and
  said here.

## Never

- A task that changes anything outside `src/`: the build, the rule, the frame around the program, the manifest,
  the bounds, the corpus, the sandbox, the proofs.
- A goal on anything but `gain_total_bytes` with its proof `sure_total_bytes`, or a gain below 0.0025.
- A task that asks to copy, port or adapt the code of an existing compressor, or names one as a model to follow
  line by line. Published descriptions of techniques are the only sources.
- A task that asks for the corpus's own text to be written into the program (it is counted, and it gains nothing:
  bytes in the program cost what they weigh).
- A pointer to anything outside this repository in a task's context.

## Publishing

The cutter publishes on `main` itself; no person reads each cut before it lands. (The architect's to confirm.)
