# The proofs

Each folder here is a candidate: its files are laid over the project's tree. `prove.py` has the judge answer each
one, and the tip against itself, and checks every answer against what is written in it (`EXPECTED`). None of this
is offered to anyone: these are texts written to be refused.

| Candidate | What it does | The judge's answer |
| --- | --- | --- |
| `tip` (no folder) | the project's own tree, against itself | 0: its figures, no gain, no proof |
| `honest-order3` | adds one model, in one new file, with no list edited | 0: a lower total, `sure_total_bytes` 1 |
| `embedded-prefix` | holds the first 65,536 bytes of the data in the program and codes nothing for them | 0, and no gain: the message is shorter, the program longer by exactly those bytes |
| `lossy` | gives the data back with its last byte changed | 1: another byte at that offset |
| `shorter` | gives the data back one byte short | 1: another length |
| `changing` | right at every run, with an address the system draws written in the message | 1: two encodings that are not the same bytes |
| `slow` | a model that takes its time | 1: stopped past its processor time |
| `hungry` | a model that keeps a gigabyte | 1: killed at its memory bound |
| `word-unsafe`, `word-extern`, `word-include`, `word-macro`, `attribute` | each breaks the project's rule on what the program may hold, to reach a file or run code before the frame | 1: the build stops at the rule, naming the line |
| `no-std-library` | names the standard library without any refused word | 1: the compiler finds no such library |
| `outside-the-program` | changes the frame around the program to read the corpus | 1: something else than the program's files |
| `finder` | **against a tip whose rule is loosened on purpose** (the standard library let in): the encoder leaves the data wherever it can write and its first 64 bytes as the message; the decoder searches every folder it can read for a longer file that starts with them | 1: nothing came back. The decode container holds no copy of the data and no file of any system, whatever a program may do |
| `calls` | **against the same loosened tip**: before it encodes and before it decodes, the frame tries to read a file of `/proc`, to look at its own program, to list the root and to ask where it is, and gives nothing back if any of it is carried out | 0, with the tip's own message (its program is heavier by some 1.3 MB: it keeps the frame of before, std-frame, with the standard library and the C library): every call was refused by the judge's gate, since one carried out would have left an empty message. This is the one project proof that fails if the program does not run behind the gate's filter (`/proc` is in every container) |

`std-frame` is no candidate: it is the frame as it was before it was made light (with the standard library and
the C library), which `prove.py` puts in the loosened tip alone, since a program that names the standard library
cannot be linked into the light frame. The loosened tip's build links it as it was then.

`same-message` is no candidate either: a script run in the sandbox, as `bee/check` is, that proves the light frame
changes no byte of the message. It builds the tree's program with the light frame and with `std-frame` (built as
`prove.py` builds the loosened tip), encodes the corpus with both, decodes both messages, and ends with 0 only if
the two messages are the same bytes (it prints their SHA-256) and a third program, the light frame with its message XORed on the way out and
back in (a right round trip, another message), does not. The judge's `message_bytes` alone could not show it.

The toy project of the judge's kind (`tools/judge/relay_toy/` in Bee-ruche/hive, with `test_relay.py`) holds the same
attacks written in C, where nothing stands between a program and the system, and one more: the control, a door left
open on purpose, through which the same cheating program is taken for right.

`slow` and `hungry` are judged with the manifest's bounds lowered (`--encode-seconds`, `--memory-mb`): the bound is
then reached in seconds, and inside the memory of the development runner (`bee/local/`). The mechanism is the same at any figure.
