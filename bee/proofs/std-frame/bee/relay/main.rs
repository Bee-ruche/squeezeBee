//! NOT the project's frame: the frame as it was before it was made light (bee/relay/main.rs, which holds neither the
//! standard library nor the C library). bee/proofs/prove.py puts it in the LOOSENED tip alone, whose program names
//! the standard library and so cannot be linked into the light frame; the build links it as it did then (without
//! -nostartfiles -nostdlib). It changes nothing the proofs against that tip show: what the judge's gate and
//! containers leave a program that has the whole system's library.
//!
//! The frame around the program, the project's own (no contribution can change it): the only code here that
//! touches a file. « run encode » and « run decode » read descriptor 3 to its end and write descriptor 4. Whoever
//! starts the program opens the two first: the judge's gate, which then lets the process open nothing; or a shell
//! (`run encode 3<data 4>message`, as bee/check does).
//!
//! The program itself (the crate `program`, built from src/) is given bytes and gives bytes back.

use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::{env, process};

fn main() {
    let phase = env::args().nth(1).unwrap_or_default();
    if phase != "encode" && phase != "decode" {
        eprintln!("usage: run encode|decode   (reads descriptor 3, writes descriptor 4)");
        process::exit(2);
    }
    // The two descriptors are this process's own, opened for it before it started, and nothing else holds them.
    let (mut given, mut wanted) = unsafe { (File::from_raw_fd(3), File::from_raw_fd(4)) };
    let mut input = Vec::new();
    if let Err(error) = given.read_to_end(&mut input) {
        eprintln!("run: descriptor 3: {error}");
        process::exit(3);
    }
    let output = if phase == "encode" { program::encode(&input) } else { program::decode(&input) };
    if let Err(error) = wanted.write_all(&output) {
        eprintln!("run: descriptor 4: {error}");
        process::exit(4);
    }
}
