#!/usr/bin/env python3
"""A STAND-IN for the corpus, made without network from text that is the hive's own.

  python3 bee/local/standin.py <folder of the hive's tree> <bytes> <seed> <file to write>

The project's corpus is 10^7 bytes of public-domain books (bee/corpus/). To build the program, prove the
judge on it and time it, something of the right size and of the nature of text is needed. This writes one: the
words of the hive's own documents and programs (every .md and .py file under the folder, in the order of their
paths), strung again by chance. Each word is drawn among those that followed the two words before it somewhere in
those files; one draw in four looks at the last word alone, so that no passage is copied for long; a draw with no
answer starts again anywhere. The draws come from Python's own generator, seeded: the same folder, size and seed
give the same bytes (Python 3.12).

It is a stand-in for plumbing and for timing. No figure measured on it says anything about the real corpus: it
has far fewer distinct words than an encyclopedia, no markup, no names, no numbers to speak of.
"""

import hashlib
import os
import random
import re
import sys


def words_of(folder):
    words = []
    for root, folders, names in os.walk(folder):
        folders.sort()
        for name in sorted(names):
            if name.endswith((".md", ".py")):
                with open(os.path.join(root, name), encoding="utf-8", errors="replace") as f:
                    words += re.findall(r"\S+|\n", f.read())
    return words


def main(arguments):
    if len(arguments) != 4:
        print(__doc__, file=sys.stderr)
        return 2
    folder, size, seed, target = arguments[0], int(arguments[1]), int(arguments[2]), arguments[3]
    words = words_of(folder)
    if len(words) < 1000:
        print("standin: too little text under that folder", file=sys.stderr)
        return 1
    after_two, after_one = {}, {}
    for i in range(len(words) - 2):
        after_two.setdefault((words[i], words[i + 1]), []).append(words[i + 2])
        after_one.setdefault(words[i + 1], []).append(words[i + 2])
    draw = random.Random(seed)
    a, b = words[0], words[1]
    made, pieces, digest = 0, [], hashlib.sha256()
    with open(target, "wb") as out:
        while made < size:
            followers = after_one.get(b) if draw.random() < 0.25 else after_two.get((a, b))
            if not followers:
                at = draw.randrange(len(words) - 2)
                a, b = words[at], words[at + 1]
                continue
            a, b = b, draw.choice(followers)
            piece = (b if b == "\n" else b + " ").encode("utf-8")
            pieces.append(piece)
            made += len(piece)
            if len(pieces) >= 65536 or made >= size:
                block = b"".join(pieces)
                if made > size:
                    block = block[:len(block) - (made - size)]
                out.write(block)
                digest.update(block)
                pieces = []
    print(f"{size} bytes, sha256 {digest.hexdigest()}, from {len(words)} words under {folder}, seed {seed}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
