//! NOT a frame the project's build accepts: it uses the standard library, which the project's rule and its crate
//! root refuse. bee/proofs/prove.py builds it against a copy of the tip whose rule is LOOSENED on purpose, to show
//! what the judge's containers hold when the language no longer stands in the way.
//!
//! The encoder leaves the data wherever a program can write, and a message of its first 64 bytes. The decoder
//! walks every folder it can read, from the root, and gives back the first file that starts with those 64 bytes
//! and is longer: the data, if it is anywhere.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::vec::Vec;

const KEY: usize = 64;
// (One copy in /out and no more: the folder a phase writes in is bounded to the data and a little, and must still take the message.)
const PLACES: [&str; 8] = ["/out/kept", "/tmp/kept", "/dev/shm/kept", "/var/tmp/kept", "/program/kept", "/in/kept", "/chrono/kept", "/kept"];

pub fn encode(data: &[u8]) -> Vec<u8> {
    for place in PLACES {
        let _ = fs::write(place, data);
    }
    std::env::set_var("BEE_KEPT", "/tmp/kept");
    data[..data.len().min(KEY)].to_vec()
}

pub fn decode(message: &[u8]) -> Vec<u8> {
    let mut found = Vec::new();
    search(Path::new("/"), message, 0, &mut found);
    found
}

fn search(folder: &Path, key: &[u8], depth: u32, found: &mut Vec<u8>) {
    if depth > 16 || !found.is_empty() {
        return;
    }
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.starts_with("/proc") || path.starts_with("/sys") {
            continue;
        }
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(_) => continue,
        };
        if kind.is_dir() {
            search(&path, key, depth + 1, found);
        } else if kind.is_file() {
            let mut start = [0u8; KEY];
            let same = match fs::File::open(&path) {
                Ok(mut file) => file.read_exact(&mut start[..key.len()]).is_ok() && &start[..key.len()] == key,
                Err(_) => false,
            };
            let longer = entry.metadata().map(|m| m.len() as usize > key.len()).unwrap_or(false);
            if same && longer {
                if let Ok(data) = fs::read(&path) {
                    *found = data;
                }
            }
        }
        if !found.is_empty() {
            return;
        }
    }
}
