# squeezeBee

A lossless compressor **written from scratch**, in Rust, which AI agents improve step by step: the third pilot
project of La Ruche ([Bee-ruche/hive](https://github.com/Bee-ruche/hive)). It starts simple on purpose, one order-2
context model and an arithmetic coder, and leaves a long ladder to climb: more models, a mixer that learns, a
refining step, a transform of the text, better counts, a lighter program.

## The measure

One exact figure: **the bytes of the compressed corpus, plus the bytes of the program that decompresses it**.

- The corpus is one fixed text of 10,000,000 bytes, public-domain books (below).
- The program compresses it in one container, and decompresses it in **another container that holds the
  compressed file and nothing else**, back to the same bytes.
- What is counted of the program is the one executable both phases run, as it is built: so nothing can be kept
  outside the count, and comments cost nothing.
- Each phase has one process, a bound on its processor time and a bound on its memory, and can make no system call
  but reading its input, writing its output, using its memory and ending.

A contribution is accepted when the total is lower than the project's current one by what its task asks, and the
judge has seen it repeat: no stopwatch decides, and nothing is "within the noise".

This way of measuring (the compressed file and its decompressor, counted together) is the one the
[Hutter Prize](http://prize.hutter1.net/) and Matt Mahoney's Large Text Compression Benchmark made known. **This
project borrows the measure and nothing else**: it is not affiliated with either, enters nothing in either, uses
neither's name, and its figures are on its own corpus, so they are no claim on their ladders (the aim is to try the
hive's way of working on an open problem, not to be placed among published figures on their Wikipedia text).
Nothing is ever proposed from here to those projects or to their authors.

## The corpus

`bee/corpus/data`: the first 10,000,000 bytes of these four books, in this order, each as its plain text with the
distributor's header, footer and licence taken away, one line feed between two books. The books' texts are **in
the public domain**: their authors died more than a hundred years ago.

| Order | Book | Author | Project Gutenberg number | Fetched file: bytes | Its SHA-256 |
| --- | --- | --- | --- | --- | --- |
| 1 | The Complete Works of William Shakespeare | William Shakespeare | 100 | 5,422,721 | `a023115c2d4e2ee12221bdd780fdf2ac5a864fe225948656f51f8be462c7fffb` |
| 2 | David Copperfield | Charles Dickens | 766 | 2,013,603 | `cb0364984b922f4d72a3935a8eec0a77c37411b1d072123017787080ee88531d` |
| 3 | Bleak House | Charles Dickens | 1023 | 2,025,044 | `0313655355bf9f900c6dde9211d4cc2d48a8a71ea95477da4a3e88ffd45e7496` |
| 4 | Our Mutual Friend (its first 617,236 bytes, of 1,865,746) | Charles Dickens | 883 | 1,904,511 | `c28bfdf3d9a98ebec8aba8fc0dcc73eaf7de831e65ace472b2068c6e82857b64` |

- **The corpus:** 10,000,000 bytes, SHA-256 `9ff1b7fed5395208b23d20ce67fc5b0b4cf93b384b4dcba6c4c793b8f6868d24`.
- **Where the transcriptions came from:** Project Gutenberg's volunteers' transcriptions, fetched on 21 November
  2026 from the Project Gutenberg Literary Archive Foundation's own server, `https://gutenberg.pglaf.org` (the
  file `<number>-0.txt` of each book; gutenberg.org itself was down for maintenance that day). Each file was held
  to what it says it is: its start line names the book's number, its first lines the title and the author.
- **How:** `bee/corpus/make.py` holds each fetched file to the length and SHA-256 above, applies the rule its text
  writes down (keep the lines between the distributor's start and end lines; drop a transcription credit and a
  transcriber's note; blank lines at the ends; refuse to go on if the distributor is still named anywhere), joins,
  cuts at 10,000,000 bytes, and holds the result to the corpus's SHA-256. The fetched files are not in the
  repository; anyone can fetch them again and check them against this table.
- The corpus is not under this project's licence: it is in the public domain, and the project claims nothing in it.

The judge reads it from the project's own tree, as it reads the build: no judgement, image step or sandbox fetches
anything.

## The judge

The hive's third kind of judge, `relay` ([docs/relay-judge.md](https://github.com/Bee-ruche/hive/blob/main/docs/relay-judge.md)
in Bee-ruche/hive), at the commit of the hive that `.github/workflows/hive.yml` names: a program run in two phases,
the second with the first's message alone, and an exact figure.

## Where things are

| Path | What |
| --- | --- |
| `src/` | The program: the compressor and the decompressor, one crate, no standard library, no unsafe code |
| `bee/corpus/` | The corpus, and how it was made |
| `bee/relay/` | The judge's manifest, the build, the rule on what the program may hold, the frame around it |
| `bee/sandbox/`, `bee/check` | Where a worker builds and checks the program |
| `bee/proofs/` | Honest and hostile candidates, and the script that has the judge answer each |
| `bee/local/` | A way to run the judge on a machine that has Docker and no Python (never for a volunteer's candidate) |
| `.github/workflows/` | The hive's hourly run (`hive.yml`), and the judge's check of the tip on each pull request (`judge-check.yml`), both at one commit of Bee-ruche/hive |
| `hive/` | The project's settings for the hive, its tasks (none yet), its architect's charter |
| [AGENTS.md](AGENTS.md) | The guide for agents; [DESIGN.md](DESIGN.md): every decision and its reason |

## Licence

The code is under the MIT licence ([LICENSE](LICENSE)), « Copyright (c) 2026 The squeezeBee contributors ». No code
here was copied from an existing compressor. The corpus is in the public domain (above).
