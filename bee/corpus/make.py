#!/usr/bin/env python3
"""How the project's corpus is made, from the public-domain books it is made of.

  python3 bee/corpus/make.py <folder of the fetched files> <file to write>

The corpus is the first 10,000,000 bytes of these books, in this order, each as its plain UTF-8 text with the
distributor's own header, footer and licence taken away (BOOKS, below). The files were fetched once, by hand, on
21 November 2026, from the Project Gutenberg Literary Archive Foundation's own server (https://gutenberg.pglaf.org,
the plain UTF-8 text of book <number>, « <number>-0.txt », in its folder), and kept as « pg<number>.txt »; they are
not in the repository. This script
runs nothing it reads: it holds each file to its recorded length and SHA-256, applies the rule, and writes the
corpus, which it holds to its own recorded length and SHA-256 (CORPUS) when those are written.

THE RULE, for each book:
1. its bytes are read as UTF-8 (a byte order mark at the start is dropped) and every line end (CR LF, or CR alone)
   becomes LF;
2. what is kept is the lines strictly between the distributor's start line (a line that begins
   « *** START OF THE PROJECT GUTENBERG EBOOK » or « *** START OF THIS PROJECT GUTENBERG EBOOK ») and its end line
   (the same with END); the start and end lines, everything before and everything after them, go;
3. then, at the head of what is kept, a paragraph that credits who produced the transcription (it begins
   « Produced by », « E-text prepared by », « Transcribed from » or « This etext was prepared by ») goes, with the
   blank lines before and after it;
4. a paragraph (lines between two blank lines) whose first line opens, after blanks and an asterisk or a bracket,
   with « Transcriber's note » (either apostrophe, any case) goes: a note of the transcription, not of the work;
5. blank lines at the head and at the end go;
6. what is left must not hold the words « Gutenberg », « gutenberg.org » or « www. », in any case: if it does, the
   script stops, and the rule is mended here, by hand, rather than the text.
The books, so treated, are joined in the order of BOOKS with one LF between two books, and the first 10,000,000
bytes of the whole, as UTF-8, are the corpus (the cut may fall inside a character: the corpus is bytes).
"""

import hashlib
import os
import re
import sys

SIZE = 10_000_000
# (Project Gutenberg's number, title, author, the fetched file's length, its SHA-256): in the corpus's order.
BOOKS = [
    (100, "The Complete Works of William Shakespeare", "William Shakespeare", 5422721, "a023115c2d4e2ee12221bdd780fdf2ac5a864fe225948656f51f8be462c7fffb"),
    (766, "David Copperfield", "Charles Dickens", 2013603, "cb0364984b922f4d72a3935a8eec0a77c37411b1d072123017787080ee88531d"),
    (1023, "Bleak House", "Charles Dickens", 2025044, "0313655355bf9f900c6dde9211d4cc2d48a8a71ea95477da4a3e88ffd45e7496"),
    (883, "Our Mutual Friend", "Charles Dickens", 1904511, "c28bfdf3d9a98ebec8aba8fc0dcc73eaf7de831e65ace472b2068c6e82857b64"),
]
CORPUS = (SIZE, "9ff1b7fed5395208b23d20ce67fc5b0b4cf93b384b4dcba6c4c793b8f6868d24")  # its length and its SHA-256
START = re.compile(r"\*\*\* ?START OF (THE|THIS) PROJECT GUTENBERG EBOOK", re.I)
END = re.compile(r"\*\*\* ?END OF (THE|THIS) PROJECT GUTENBERG EBOOK", re.I)
CREDIT = re.compile(r"(Produced by|E-text prepared by|Transcribed from|This etext was prepared by)", re.I)
NOTE = re.compile(r"\s*[*\[]?\s*transcriber['" + chr(0x2019) + r"]s note", re.I)  # (either apostrophe)
FORBIDDEN = re.compile(r"gutenberg|www\.", re.I)


class Broken(Exception):
    pass


def works_of(number, raw):
    """The work alone, from one fetched file (the rule, steps 1 to 6)."""
    text = raw.decode("utf-8")
    if text.startswith(chr(0xFEFF)):  # a byte order mark
        text = text[1:]
    lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    starts = [i for i, line in enumerate(lines) if START.match(line)]
    ends = [i for i, line in enumerate(lines) if END.match(line)]
    if len(starts) != 1 or len(ends) != 1 or ends[0] <= starts[0]:
        raise Broken(f"pg{number}: not one start line and one end line after it ({len(starts)} and {len(ends)})")
    kept = lines[starts[0] + 1:ends[0]]
    while kept and not kept[0].strip():
        kept.pop(0)
    if kept and CREDIT.match(kept[0]):
        while kept and kept[0].strip():
            kept.pop(0)
    # (step 4) a transcriber's note goes, with the blank lines after it; those before it keep the paragraphs apart
    notes_gone, at = [], 0
    while at < len(kept):
        if kept[at].strip() and (at == 0 or not kept[at - 1].strip()) and NOTE.match(kept[at]):
            while at < len(kept) and kept[at].strip():
                at += 1
            while at < len(kept) and not kept[at].strip():
                at += 1
            continue
        notes_gone.append(kept[at])
        at += 1
    kept = notes_gone
    while kept and not kept[0].strip():
        kept.pop(0)
    while kept and not kept[-1].strip():
        kept.pop()
    for at, line in enumerate(kept):
        if FORBIDDEN.search(line):
            raise Broken(f"pg{number}: line {at + 1} of what is kept still names the distributor: mend the rule")
    return "\n".join(kept)


def main(arguments):
    if len(arguments) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    folder, target = arguments
    works, recorded = [], True
    try:
        for number, title, author, length, fingerprint in BOOKS:
            with open(os.path.join(folder, f"pg{number}.txt"), "rb") as f:
                raw = f.read()
            found = (len(raw), hashlib.sha256(raw).hexdigest())
            if length is None:
                recorded = False
            elif found != (length, fingerprint):
                raise Broken(f"pg{number}: {found[0]} bytes, SHA-256 {found[1]}: not the file recorded")
            work = works_of(number, raw)
            works.append(work)
            print(f"pg{number}\t{found[0]}\t{found[1]}\t{len(work.encode('utf-8'))}\t{author}, {title}")
        whole = "\n".join(works).encode("utf-8")
        if len(whole) < SIZE:
            raise Broken(f"the books hold {len(whole)} bytes, fewer than {SIZE}: add a book to BOOKS")
        corpus = whole[:SIZE]
        made = (len(corpus), hashlib.sha256(corpus).hexdigest())
        if CORPUS[1] is not None and made != CORPUS:
            raise Broken(f"the corpus made is {made[0]} bytes, SHA-256 {made[1]}: not the corpus recorded")
    except (Broken, OSError, UnicodeDecodeError) as broken:
        print(f"make: {broken}", file=sys.stderr)
        return 1
    with open(target, "wb") as f:
        f.write(corpus)
    print(f"corpus\t{made[0]}\t{made[1]}\t(of {len(whole)} bytes of the books){'' if recorded else '  (the fetched files are not recorded yet)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
