#!/usr/bin/env python3
"""The project's rule on what the program's files may hold, and the crate root the build compiles.

  python3 bee/relay/rules.py <the program's folder (src)> <where the crate root is written>

Run by bee/relay/build, in the build's container, before the compiler sees anything. It reads the files as text
and runs none of it. A file that breaks the rule stops the build, with the file, the line and the reason.

The rule (AGENTS.md says the same to who writes the program):
- the program is `src/<name>.rs` files and `src/models/<name>.rs` files, and nothing else; a name is lower-case
  letters, digits and underscores, starts with a letter, and is no word of the language;
- printable ASCII and line feeds: no tab, no carriage return, no other byte;
- none of the words in WORDS, anywhere, comments and strings included: nothing is parsed, so nothing can hide;
- the character « # » only to open one of the attributes in ATTRIBUTE.
With the crate root below (no standard library, no unsafe code) the language then gives a file of the program
nothing but computing and allocating memory: no file, no clock, no thread, no code or bytes from outside the files
that are built.

That is a rule of good order, as strong as the compiler is sound and no stronger: the compiler has known holes
through which code that looks safe is not, and this text does not see them. So the judge does not rest on it. It
takes every program for native code that can make any system call, and shuts the calls themselves: behind its gate
a phase can read and write the two descriptors it is handed, use its own memory and end, and nothing else, in a
container of its own whose image holds no file. What the rule buys is a program that is the same on every machine,
and bees that are told at once, with a line number, what they may not write.
"""

import os
import re
import sys

NAME = re.compile(r"[a-z][a-z0-9_]{0,30}")
# Words of the language, and names the crate root uses itself: no file takes one as its module's name.
TAKEN = frozenset("""as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub
ref return self static struct super trait true type unsafe use where while abstract become box do final macro override priv try typeof
unsized virtual yield union alloc core std models program main test""".split())
# unsafe, extern: code the compiler does not check, and anything outside the two libraries the root allows (core, alloc);
# macro_rules, macro: a macro can spell a word or an attribute the rule would not see; include…, env…: bytes from outside
# the counted files, taken in when the program is built; asm…: instructions written by hand; concat_idents: a spelled word.
WORDS = ("unsafe", "extern", "macro_rules", "macro", "include", "include_str", "include_bytes", "env", "option_env",
         "concat_idents", "asm", "global_asm", "naked_asm")
WORD = re.compile(r"(?<![A-Za-z0-9_])(?:" + "|".join(WORDS) + r")(?![A-Za-z0-9_])")
TRAITS = r"(?:Clone|Copy|Debug|Default|PartialEq|Eq|PartialOrd|Ord|Hash)"
ATTRIBUTE = re.compile(r"#\[(?:inline|inline\(always\)|inline\(never\)|derive\(" + TRAITS + r"(?:, " + TRAITS + r")*\))\]")
REQUIRED = ("context", "frame")  # the root names them: the models are told the context, the frame is what the build calls
ROOT_HEAD = """// Written by bee/relay/rules.py when the program is built: the crate's root is the project's own, and no file of
// the program is the root. The program has the language's core and its allocator, and nothing else.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
"""


class Broken(Exception):
    pass


def names_in(folder, what):
    """The module names of a folder's files, sorted; anything that is not `<name>.rs` breaks the rule."""
    names = []
    for entry in sorted(os.listdir(folder)):
        path = os.path.join(folder, entry)
        if os.path.islink(path):
            raise Broken(f"{what}/{entry}: a link")
        if os.path.isdir(path):
            if what == "src" and entry == "models":
                continue
            raise Broken(f"{what}/{entry}: a folder (the program is src/<name>.rs and src/models/<name>.rs)")
        name, dot, extension = entry.rpartition(".")
        if dot != "." or extension != "rs" or not NAME.fullmatch(name) or name in TAKEN:
            raise Broken(f"{what}/{entry}: not <name>.rs with a name of lower-case letters, digits and underscores that is no word of the language")
        names.append(name)
    return names


def check(path, shown):
    with open(path, "rb") as f:
        data = f.read()
    for number, line in enumerate(data.split(b"\n"), 1):
        if any(byte < 0x20 or byte > 0x7E for byte in line):
            raise Broken(f"{shown}:{number}: a byte that is not printable ASCII (no tab, no carriage return, no accent)")
        text = line.decode("ascii")
        found = WORD.search(text)
        if found:
            raise Broken(f"{shown}:{number}: the word « {found.group(0)} » (the program may not hold it, even in a comment)")
        at = text.find("#")
        while at >= 0:
            attribute = ATTRIBUTE.match(text, at)
            if not attribute:
                raise Broken(f"{shown}:{number}: the character « # » opens something else than #[inline], #[inline(always)], "
                             "#[inline(never)] or #[derive(…)] of the language's own traits")
            at = text.find("#", attribute.end())


def root(source, modules, models):
    lines = [ROOT_HEAD]
    lines += [f'#[path = "{source}/{name}.rs"]\npub mod {name};' for name in modules]
    lines.append("\n/// The models, one per file of src/models, in the order of their names.\npub mod models {")
    lines += [f'    #[path = "{source}/models/{name}.rs"]\n    pub mod {name};' for name in models]
    lines.append("    use crate::context::Context;\n")
    lines.append("    /// How many predictions the models make together at each bit.")
    lines.append("    pub const INPUTS: usize = " + " + ".join(f"{name}::OUTPUTS" for name in models) + ";\n")
    lines.append("    pub struct Set {")
    lines += [f"        {name}: {name}::Model," for name in models]
    lines.append("    }\n\n    impl Set {\n        pub fn new() -> Self {")
    lines.append("            Set { " + ", ".join(f"{name}: {name}::Model::new()" for name in models) + " }")
    lines.append("        }\n\n        /// Every model's predictions for the bit to come, one after the other in `out`.")
    lines.append("        pub fn predict(&mut self, cx: &Context, history: &[u8], out: &mut [i32]) {\n            let mut at = 0;")
    for name in models:
        lines.append(f"            self.{name}.predict(cx, history, &mut out[at..at + {name}::OUTPUTS]);\n            at += {name}::OUTPUTS;")
    lines.append("            let _ = at;\n        }\n\n        /// Every model is told the bit that came.")
    lines.append("        pub fn update(&mut self, cx: &Context, bit: u32) {")
    lines += [f"            self.{name}.update(cx, bit);" for name in models]
    lines.append("        }\n    }\n}\n\npub use frame::{decode, encode};")
    return "\n".join(lines) + "\n"


def main(arguments):
    if len(arguments) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    source, target = os.path.abspath(arguments[0]), arguments[1]
    try:
        if not os.path.isdir(source) or os.path.islink(source):
            raise Broken("src: the program's folder is missing")
        modules = names_in(source, "src")
        folder = os.path.join(source, "models")
        if os.path.islink(folder) or not os.path.isdir(folder):
            raise Broken("src/models: the models' folder is missing")
        models = names_in(folder, "src/models")
        for needed in REQUIRED:
            if needed not in modules:
                raise Broken(f"src/{needed}.rs is missing")
        if not models:
            raise Broken("src/models holds no model")
        for name in modules:
            check(os.path.join(source, name + ".rs"), f"src/{name}.rs")
        for name in models:
            check(os.path.join(folder, name + ".rs"), f"src/models/{name}.rs")
    except (Broken, OSError) as broken:
        print(f"rules: {broken}", file=sys.stderr)
        return 1
    with open(target, "w", encoding="ascii") as f:
        f.write(root(source, modules, models))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
