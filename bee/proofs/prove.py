#!/usr/bin/env python3
"""The project's proofs: its own tip and a set of candidates, honest and hostile, judged by the relay judge.

  python3 bee/proofs/prove.py --kind FOLDER --hive FOLDER --project FOLDER --work FOLDER [--out FILE]
                              [--bytes N [--seed 20261121]] [--encode-seconds N] [--decode-seconds N]
                              [--memory-mb N] [--image-only] [name …]

  --kind     a folder that holds the hive's tools/judge with relay.py in it
  --hive     the hive's tree (with --bytes: the stand-in data is made of its text, bee/local/standin.py)
  --project  this project's tree
  --work     where trees and reports are made (a folder of the machine that judges, not a slow shared one)

It runs where the judge runs (root or sudo, Docker): in the development runner (bee/local/run.sh), or on a Linux
machine that has them. Nothing of a candidate runs here: the judge builds and runs them in its containers.

The tip judged here is the project's own tree, its corpus included (bee/corpus/data), with, when asked, other
bounds in its manifest (--encode-seconds, --decode-seconds, --memory-mb: to prove a bound without waiting minutes or
needing more memory than the runner has). With --bytes, its manifest names a STAND-IN instead of the corpus
(bee/local/standin.py: that many bytes of text made from the hive's own files), for timings at another size.
Everything else is the project's, byte for byte.

Each name below is a folder of bee/proofs/ whose files are laid over the tip's to make a candidate. What is expected
of each is written here, and checked: the script ends with 0 only if every judgement gave what was expected.
"""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PREFIX_BYTES = 65536

# name: (the tip it is judged against, the code expected, what the reason must say or what the figures must show)
EXPECTED = {
    "tip": ("tip", 0, lambda f: f["sure_total_bytes"] == 0 and f["gain_total_bytes"] == 0.0 and f["runs"] == 3),
    "honest-order3": ("tip", 0, lambda f: f["sure_total_bytes"] == 1 and f["gain_total_bytes"] > 0.01 and f["runs"] == 7),
    "embedded-prefix": ("tip", 0, lambda f: f["sure_total_bytes"] == 0 and f["gain_total_bytes"] < 0 and f["message_bytes"] < f["tip_total_bytes"]),
    "lossy": ("tip", 1, r"first decoding does not give the data back: the same length, another byte at offset \d+"),
    "shorter": ("tip", 1, r"first decoding does not give the data back: \d+ bytes where the data holds \d+"),
    "changing": ("tip", 1, r"two encodings of the same data are not the same bytes"),
    "slow": ("tip", 1, r"first encoding: stopped after \d+\.\d s of processor time; its bound is \d+ s"),
    "hungry": ("tip", 1, r"first encoding: ended by signal 9 \(killed: past its memory bound, most likely\)"),
    "word-unsafe": ("tip", 1, r"the candidate's build: exit status 1\..*rules: src/models/peek\.rs:\d+: the word « unsafe »"),
    "word-extern": ("tip", 1, r"the candidate's build: exit status 1\..*rules: src/models/peek\.rs:\d+: the word « extern »"),
    "word-include": ("tip", 1, r"the candidate's build: exit status 1\..*rules: src/models/peek\.rs:\d+: the word « include_bytes »"),
    "word-macro": ("tip", 1, r"the candidate's build: exit status 1\..*rules: src/models/peek\.rs:\d+: the word « macro_rules »"),
    "attribute": ("tip", 1, r"the candidate's build: exit status 1\..*rules: src/models/peek\.rs:\d+: the character « # »"),
    "no-std-library": ("tip", 1, r"the candidate's build: exit status 1\."),
    "outside-the-program": ("tip", 1, r"the candidate changes something else than the program's files"),
    "finder": ("loosened", 1, r"first decoding does not give the data back: 0 bytes where the data holds \d+"),
    "calls": ("loosened", 0, lambda f: f["sure_total_bytes"] == 0 and f["runs"] == 7),  # (right: so every call was refused; its program is a little heavier)
}


def run(command, **options):
    return subprocess.run(command, stdin=subprocess.DEVNULL, **options)


def copy_tree(source, target, skip=()):
    for root, folders, names in os.walk(source):
        relative = os.path.relpath(root, source)
        folders[:] = sorted(f for f in folders if os.path.normpath(os.path.join(relative, f)) not in skip)
        os.makedirs(os.path.join(target, relative), exist_ok=True)
        for name in sorted(names):
            if os.path.normpath(os.path.join(relative, name)) in skip:
                continue
            shutil.copyfile(os.path.join(root, name), os.path.join(target, relative, name))
            os.chmod(os.path.join(target, relative, name), 0o644)
    for root, _, _ in os.walk(target):
        os.chmod(root, 0o755)


def lay_over(tree, overlay):
    for root, _, names in os.walk(overlay):
        for name in names:
            relative = os.path.relpath(os.path.join(root, name), overlay)
            os.makedirs(os.path.dirname(os.path.join(tree, relative)), exist_ok=True)
            shutil.copyfile(os.path.join(root, name), os.path.join(tree, relative))
            os.chmod(os.path.join(tree, relative), 0o644)
    for root, _, _ in os.walk(tree):
        os.chmod(root, 0o755)


def prefix_source(data):
    """src/prefix.rs: the data's first bytes as a constant of the program, every byte written as an escape."""
    lines = ["//! The first bytes of the data, held by the program itself (written by bee/proofs/prove.py).", "",
             "pub static PREFIX: &[u8] = b\"\\"]
    for at in range(0, len(data), 24):
        lines.append("".join(f"\\x{byte:02x}" for byte in data[at:at + 24]) + "\\")
    return "\n".join(lines) + "\n\";\n"


def loosened(text):
    """The project's rule with its two walls taken down: the crate root lets the standard library in, and no word is
    refused. ONLY for the proof that the judge's containers hold nothing to find, whatever a program may do."""
    less = text.replace("#![no_std]\n#![forbid(unsafe_code)]\n", "")
    less = re.sub(r"WORDS = \(.*?\)\n", 'WORDS = ("a_word_nobody_writes",)\n', less, count=1, flags=re.S)
    if less.count("#![no_std]") or "a_word_nobody_writes" not in less:
        raise SystemExit("prove: the rule's text is not what this script loosens")
    return less


def main():
    parser = argparse.ArgumentParser(description="The project's proofs, through the relay judge.")
    for name in ("kind", "hive", "project", "work"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--out")
    parser.add_argument("--bytes", type=int)
    parser.add_argument("--seed", type=int, default=20261121)
    parser.add_argument("--encode-seconds", type=int)
    parser.add_argument("--decode-seconds", type=int)
    parser.add_argument("--memory-mb", type=int)
    parser.add_argument("--image-only", action="store_true", help="make the judge's image for the tip, and stop (the one step with network)")
    parser.add_argument("names", nargs="*")
    args = parser.parse_args()
    names = args.names or list(EXPECTED)
    unknown = [name for name in names if name not in EXPECTED]
    if unknown:
        parser.error("no such proof: " + ", ".join(unknown))
    relay = os.path.join(args.kind, "relay.py")
    os.makedirs(args.work, exist_ok=True)

    # The tip: the project's tree, its corpus or (--bytes) a stand-in, and, when asked, other bounds.
    tip = os.path.join(args.work, "tip")
    shutil.rmtree(tip, ignore_errors=True)
    copy_tree(args.project, tip, skip=(".git",))
    with open(os.path.join(tip, "bee", "relay", "project.json"), encoding="utf-8") as f:
        manifest = json.load(f)
    if args.bytes is not None:
        data_path = os.path.join(tip, "bee", "local", "standin")
        made = run([sys.executable, os.path.join(args.project, "bee", "local", "standin.py"), args.hive, str(args.bytes), str(args.seed), data_path],
                   capture_output=True, text=True)
        if made.returncode != 0:
            raise SystemExit("prove: the stand-in could not be made: " + made.stderr)
        print("stand-in:", made.stdout.strip(), flush=True)
        with open(data_path, "rb") as f:
            data = f.read()
        manifest["data"] = {"path": "bee/local/standin", "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
    else:
        with open(os.path.join(tip, *manifest["data"]["path"].split("/")), "rb") as f:
            data = f.read()
        print(f"corpus: {len(data)} bytes, sha256 {hashlib.sha256(data).hexdigest()} (the manifest says {manifest['data']['sha256']})", flush=True)
    for key, value in (("encode_seconds", args.encode_seconds), ("decode_seconds", args.decode_seconds), ("memory_mb", args.memory_mb)):
        if value is not None:
            manifest[key] = value
    with open(os.path.join(tip, "bee", "relay", "project.json"), "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    print("bounds:", {key: manifest[key] for key in ("encode_seconds", "decode_seconds", "memory_mb", "build_seconds")}, flush=True)
    if args.image_only:
        return run([sys.executable, "-B", relay, "--tip", tip, "--image-only"]).returncode

    tips = {"tip": tip}
    if any(EXPECTED[name][0] == "loosened" for name in names):
        tips["loosened"] = os.path.join(args.work, "loosened")
        shutil.rmtree(tips["loosened"], ignore_errors=True)
        copy_tree(tip, tips["loosened"])
        rule = os.path.join(tips["loosened"], "bee", "relay", "rules.py")
        with open(rule, encoding="utf-8") as f:
            text = f.read()
        with open(rule, "w", encoding="utf-8") as f:
            f.write(loosened(text))
        # A program that names the standard library cannot be linked into the light frame (no standard library, no
        # C library, its own start): the loosened tip keeps the frame as it was (bee/proofs/std-frame), linked as it
        # was, without the light frame's link arguments and as a position-independent program.
        lay_over(tips["loosened"], os.path.join(HERE, "std-frame"))
        build = os.path.join(tips["loosened"], "bee", "relay", "build")
        with open(build, encoding="utf-8") as f:
            text = f.read()
        if text.count(" $alone ") != 1 or text.count(" -C relocation-model=static") != 1:
            raise SystemExit("prove: the build's text is not what this script loosens")
        with open(build, "w", encoding="utf-8") as f:
            f.write(text.replace(" $alone ", " ").replace(" -C relocation-model=static", ""))

    results, failed = {}, []
    for name in names:
        base, code, check = EXPECTED[name]
        report_path = os.path.join(args.work, f"report-{name}.json")
        command = [sys.executable, "-B", relay, "--tip", tips[base], "--budget", "3480", "--json", report_path,
                   "--workdir", os.path.join(args.work, "judging"), "--task-file", "src/"]
        if name == "tip":
            command.append("--tip-itself")
        else:
            candidate = os.path.join(args.work, "candidate-" + name)
            shutil.rmtree(candidate, ignore_errors=True)
            copy_tree(tips[base], candidate)
            lay_over(candidate, os.path.join(HERE, name))
            if name == "embedded-prefix":
                with open(os.path.join(candidate, "src", "prefix.rs"), "w", encoding="ascii") as f:
                    f.write(prefix_source(data[:PREFIX_BYTES]))
            command += ["--candidate", candidate]
        ran = run(command, capture_output=True, text=True)
        with open(report_path, encoding="utf-8") as f:
            report = json.load(f)
        figures = {key: value for key, value in report.items() if key not in ("kind", "code", "reason", "step")}
        if report["code"] != ran.returncode or ran.returncode != code:
            as_expected = False
        elif code == 0:
            as_expected = bool(check(figures))
        else:
            as_expected = re.search(check, report["reason"], re.S) is not None
        results[name] = {"against": base, "code": ran.returncode, "expected_code": code, "as_expected": as_expected,
                         "reason": report.get("reason"), "figures": figures, "step": report.get("step")}
        if not as_expected:
            failed.append(name)
        said = json.dumps(figures, sort_keys=True) if ran.returncode == 0 else " ".join((report.get("reason") or "").split())[:420]
        print(f"{'ok  ' if as_expected else 'FAIL'} {name:20s} code {ran.returncode} (expected {code})  {said}", flush=True)
        if name != "tip":
            shutil.rmtree(candidate, ignore_errors=True)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            json.dump({"bytes": args.bytes, "seed": args.seed, "data_sha256": manifest["data"]["sha256"],
                       "bounds": {key: manifest[key] for key in ("encode_seconds", "decode_seconds", "memory_mb")}, "results": results}, f, indent=1, sort_keys=True)
    print("every judgement gave what was expected" if not failed else "NOT as expected: " + ", ".join(failed))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
