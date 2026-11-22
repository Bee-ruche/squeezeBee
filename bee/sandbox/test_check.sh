#!/usr/bin/env bash
# Proves that bee/check, run in bee/sandbox/sans-docker.sh, checks with the judge's own toolchain in /usr/bin or not at
# all (exit status 3, and why): what another folder of the PATH holds is neither built with nor a reason to stop. It
# takes that toolchain apart on the machine, one piece at a time, puts tools that fail if called earlier on the PATH,
# and puts everything back.
#
#   test_check.sh --on-a-throwaway-machine
#
# ONLY ON A THROWAWAY MACHINE, AS ROOT: it moves files of /usr aside (rustc, cc, python3, the static C library) and
# writes stand-ins in /usr/bin and /usr/local/bin, and puts everything back after each case and when it ends; killed
# outright, it leaves what it moved aside under <name>.bee-test-aside. It was written for a container made for it
# (bee/README.md, "The sandbox"). It runs the launcher and the check of the work tree it sits in, whose machine must
# hold the judge's toolchain in /usr to begin with. Needs bash and git.
# Exit status: 0 if no check failed.
set -u
[ "${1:-}" = --on-a-throwaway-machine ] || { sed -n '2,13p' "$0" >&2; exit 2; }
[ "$(id -u)" = 0 ] || { echo "test_check.sh: root is needed: it moves files of /usr" >&2; exit 2; }
here=$(realpath -- "${BASH_SOURCE[0]}"); repo=${here%/bee/sandbox/*}
cd "$repo" || exit 2
for f in /usr/local/bin/rustc /usr/local/bin/cc /usr/local/bin/as /usr/local/bin/ld /usr/local/bin/python3 "$repo/libc.a"; do
  [ ! -e "$f" ] && [ ! -L "$f" ] || { echo "test_check.sh: $f exists, and this script would write it: not run" >&2; exit 2; }
done
npass=0; nfail=0
verdict() { if [ "$3" -eq 0 ]; then printf 'PASS  %-3s %s\n' "$1" "$2"; npass=$((npass + 1)); else printf 'FAIL  %-3s %s   <-- %s\n' "$1" "$2" "$4"; nfail=$((nfail + 1)); fi; }
first_line() { printf '%s' "$1" | grep -m1 -E '^check: ' | cut -c1-150; }
aside=(); made=()
put_aside() { mv -T -- "$1" "$1.bee-test-aside" && aside+=("$1"); }
stand_in() { printf '#!/bin/sh\n%s\n' "$2" >"$1" && chmod 755 "$1" && made+=("$1"); }
# A tool that answers a version and fails at any real work: were the check or the build to call it, they would fail.
broken() { stand_in "$1" "case \"\$1\" in --version|-V) echo \"$2\"; exit 0 ;; esac; echo \"$1 is a stand-in: not to be built with\" >&2; exit 1"; }
restore() {
  local f
  for f in ${made[@]+"${made[@]}"}; do rm -f -- "$f"; done
  for f in ${aside[@]+"${aside[@]}"}; do [ ! -e "$f.bee-test-aside" ] && [ ! -L "$f.bee-test-aside" ] || mv -T -- "$f.bee-test-aside" "$f"; done
  rm -rf /root/.cargo-bee-test; aside=(); made=()
}
trap restore EXIT
# check [caller's PATH]: the check in the launcher, as a worker runs it; sets $out and $rc.
check() { out=$(PATH=${1:-$PATH} bee/sandbox/sans-docker.sh bee/check 100000 2>&1); rc=$?; }
figures() { grep -E '^(program_bytes|message_bytes|total_bytes) ' <<<"$out"; }
# stops <id> <description> <a word its reason must hold>: the check must stop with 3, for that reason.
stops() { check "${4:-}"; verdict "$1" "$2" "$([ "$rc" -eq 3 ] && grep -q -- "$3" <<<"$out" && grep -q ': stop\.$' <<<"$out" && ! grep -q '^program_bytes' <<<"$out"; echo $?)" "rc $rc: $(first_line "$out")"; restore; }
# runs <id> <description>: the check must run with the judge's tools and give t0's figures, to the byte.
runs() { check; verdict "$1" "$2" "$([ "$rc" -eq 0 ] && grep -q '^round trip: the same bytes' <<<"$out" && [ "$(figures)" = "$base" ]; echo $?)" "rc $rc: $(first_line "$out") $(figures | tr '\n' ' ')"; restore; }

echo "work tree: $repo"
echo "machine: $(uname -r); $(/usr/bin/rustc --version 2>&1); cc: $(readlink -f /usr/bin/cc); as: $(PATH=/usr/bin:/bin /usr/bin/cc -print-prog-name=as); ld: $(PATH=/usr/bin:/bin /usr/bin/cc -print-prog-name=ld); collect2: $(/usr/bin/cc -print-prog-name=collect2)"
libc=$(/usr/bin/cc -print-file-name=libc.a)
check
base=$(figures)
verdict t0 "with the judge's toolchain in /usr/bin, the check runs and the round trip holds" "$([ "$rc" -eq 0 ] && grep -q '^round trip: the same bytes' <<<"$out" && [ -n "$base" ]; echo $?)" "rc $rc: $(printf '%s' "$out" | tail -n 1)"
[ "$rc" -eq 0 ] || { echo "the machine does not hold the judge's toolchain to begin with: nothing more is tried" >&2; exit 1; }
status_before=$(git status --porcelain --ignored)

# What a cloud machine had: another python3 and another rustc in /usr/local/bin, which comes first on the launcher's
# PATH. The check names the judge's by where they are: it runs, with them.
broken /usr/local/bin/python3 "Python 3.11.0 (a stand-in)"; broken /usr/local/bin/rustc "rustc 1.80.0 (a stand-in)"
runs t1 "another python3 and another rustc in /usr/local/bin, first on the launcher's PATH, the judge's in /usr/bin: the check runs with the judge's, t0's figures"
# The linker's side: gcc calls the assembler and the linker through the PATH.
broken /usr/local/bin/cc "cc (a stand-in)"; broken /usr/local/bin/as "as (a stand-in)"; broken /usr/local/bin/ld "ld (a stand-in)"
runs t2 "another cc, as and ld in /usr/local/bin, first on the launcher's PATH: the check runs with /usr/bin's, t0's figures"
put_aside /usr/bin/rustc; stand_in /usr/bin/rustc 'echo "rustc 1.80.0 (a stand-in)"'
stops t3 "/usr/bin/rustc of another release: stop" "where the judge's is rustc 1.75"
# The judge's own rustc, moved out of /usr and named from root's home folder, first on the caller's PATH: outside it
# runs (t4a, the case's setting); the check, which looks for /usr/bin/rustc, does not take it (t4).
put_aside /usr/bin/rustc; mkdir -p /root/.cargo-bee-test/bin; ln -s /usr/bin/rustc.bee-test-aside /root/.cargo-bee-test/bin/rustc
home_rustc=$(PATH=/root/.cargo-bee-test/bin:$PATH rustc --version 2>&1)
verdict t4a "(t4's setting) outside, the caller's PATH finds a working rustc 1.75 in a home folder, and none is in /usr" "$(case "$home_rustc" in "rustc 1.75."*) [ ! -e /usr/bin/rustc ]; echo $?;; *) echo 1;; esac)" "$home_rustc"
stops t4 "that rustc is not the judge's: no /usr/bin/rustc, stop" "no /usr/bin/rustc" "/root/.cargo-bee-test/bin:$PATH"
put_aside "$libc"
stops t5 "no static C library: stop" "no static C library"
put_aside "$libc"; echo "a libc.a a candidate left" >"$repo/libc.a"; made+=("$repo/libc.a")
stops t6 "no static C library, and a file named libc.a at the root of the work tree (gcc's bare answer would name it): stop" "gcc knows none"
put_aside /usr/bin/cc; stand_in /usr/local/bin/cc 'exec /usr/bin/cc.bee-test-aside "$@"'
stops t7 "no /usr/bin/cc, a working one in /usr/local/bin: stop" "no /usr/bin/cc"
real_python=$(readlink -f /usr/bin/python3); put_aside /usr/bin/python3; stand_in /usr/local/bin/python3 "exec $real_python \"\$@\""
stops t8 "no /usr/bin/python3, a working one in /usr/local/bin: stop" "no /usr/bin/python3"

check
verdict t9 "everything put back: the check runs again, with t0's figures, and the clone is as it was" "$([ "$rc" -eq 0 ] && [ "$(figures)" = "$base" ] && [ "$(git status --porcelain --ignored)" = "$status_before" ]; echo $?)" "rc $rc; git status: $(git status --porcelain --ignored | tr '\n' ' ')"
echo
echo "result: $npass passed, $nfail failed"
[ "$nfail" -eq 0 ]
