//! The frame around the program, the project's own (no contribution can change it): the only code here that
//! touches a file. « run encode » and « run decode » read descriptor 3 to its end and write descriptor 4. Whoever
//! starts the program opens the two first: the judge's gate, which then lets the process open nothing; or a shell
//! (`run encode 3<data 4>message`, as bee/check does).
//!
//! The program itself (the crate `program`, built from src/) is given bytes and gives bytes back.
//!
//! This frame holds neither the language's standard library nor the C library: everything the program needs from
//! the system is written here, and is little. What `std` and the C library did for the program, and this frame does
//! itself:
//! - the process's start (`_start`, below): it aligns the stack, reads the arguments the kernel left on it, and
//!   calls `start`; nothing else is set up, since nothing here needs threads, thread-local storage or environment;
//! - the system calls, by the instruction itself (`syscall`). The ones made, and nothing else: read, write,
//!   rt_sigaction (SIGPIPE ignored, as `std` does before `main`), mmap, munmap, mremap, exit_group, and getpid,
//!   gettid and tgkill (SIGABRT to itself, when memory could not be had). All are among those the judge's gate lets
//!   through (`tools/judge/gate.c` in Bee-ruche/hive);
//! - the memory allocator (`Heap`), on mmap, munmap and mremap;
//! - memcpy, memmove, memset, memcmp and bcmp, which the compiler calls for copies and comparisons;
//! - what a panic does: one line on descriptor 2 (where the program panicked, and why), then the exit status 101;
//!   or, when the panic comes from memory that could not be had, SIGABRT, as a C library's abort().
//!
//! Exit status: 0; 2 the usage; 3 descriptor 3 could not be read; 4 descriptor 4 could not be written; 101 the
//! program panicked; ended by SIGABRT: memory could not be had. Nothing is written on descriptor 2 but one line, and
//! only on those errors.
//!
//! The build links it with `-nostartfiles -nostdlib -static`, as a position-dependent program: nothing relocates it,
//! so no address in it may need relocating (bee/relay/build).

#![no_std]
#![no_main]
// No function of this file may be turned by the compiler into a call to memcpy, memset, memcmp or bcmp: those are
// written here, and a loop of theirs so turned would call itself. (Copies and fills are an instruction anyway.)
#![no_builtins]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::{asm, global_asm};
use core::cell::UnsafeCell;
use core::panic::PanicInfo;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

// The start. The kernel leaves the stack pointer on the argument count, followed by the arguments' addresses; it
// is aligned to 16 bytes at entry, and aligned again here all the same, so that `start` is entered as any function
// is (the stack at 16 bytes, less the return address `call` pushes).
global_asm!(
    ".globl _start",
    ".type _start, @function",
    "_start:",
    "xor ebp, ebp",
    "mov rdi, rsp",
    "and rsp, -16",
    "call {start}",
    "ud2",
    start = sym start,
);

unsafe extern "C" fn start(stack: *const usize) -> ! {
    // First, as std does before `main`: a write to a pipe whose reader is gone is then an error of that write (the
    // usage's status 2, or 4), not a signal.
    let ignore = SigAction { handler: SIG_IGN, flags: 0, restorer: 0, mask: 0 };
    syscall4(RT_SIGACTION, SIGPIPE, &ignore as *const SigAction as usize, 0, 8);
    let count = *stack;
    let arguments = stack.add(1) as *const *const u8;
    let phase = if count > 1 { terminated(*arguments.add(1)) } else { &[][..] };
    let encoding = phase == b"encode";
    if !encoding && phase != b"decode" {
        say(b"usage: run encode|decode   (reads descriptor 3, writes descriptor 4)\n");
        exit(2);
    }
    // The two descriptors are this process's own, opened for it before it started, and nothing else holds them.
    let input = match read_all(3) {
        Ok(input) => input,
        Err(errno) => failed(b"run: descriptor 3: os error ", errno, 3),
    };
    let output = if encoding { program::encode(&input) } else { program::decode(&input) };
    match write_all(4, &output) {
        Ok(()) => exit(0),
        Err(0) => {
            say(b"run: descriptor 4: failed to write whole buffer\n");
            exit(4)
        }
        Err(errno) => failed(b"run: descriptor 4: os error ", errno, 4),
    }
}

/// The bytes of a string the kernel gave, up to its terminating zero.
unsafe fn terminated(at: *const u8) -> &'static [u8] {
    let mut length = 0;
    while *at.add(length) != 0 {
        length += 1;
    }
    core::slice::from_raw_parts(at, length)
}

/// Descriptor `fd` read to its end; or the system's error number.
fn read_all(fd: usize) -> Result<Vec<u8>, usize> {
    let mut data: Vec<u8> = Vec::new();
    loop {
        if data.capacity() == data.len() {
            data.reserve(core::cmp::max(data.len(), 1 << 16));
        }
        let room = data.capacity() - data.len();
        let got = unsafe { syscall3(READ, fd, data.as_mut_ptr().add(data.len()) as usize, room) };
        if got == 0 {
            return Ok(data);
        }
        if got == -EINTR {
            continue;
        }
        if got < 0 {
            return Err(-got as usize);
        }
        unsafe { data.set_len(data.len() + got as usize) };
    }
}

/// All of `bytes` written to descriptor `fd`; or the system's error number, 0 when a write wrote nothing.
fn write_all(fd: usize, mut bytes: &[u8]) -> Result<(), usize> {
    while !bytes.is_empty() {
        let wrote = unsafe { syscall3(WRITE, fd, bytes.as_ptr() as usize, core::cmp::min(bytes.len(), isize::MAX as usize)) };
        if wrote == -EINTR {
            continue;
        }
        if wrote <= 0 {
            return Err(-wrote as usize);
        }
        bytes = &bytes[wrote as usize..];
    }
    Ok(())
}

/// One line on descriptor 2, whatever becomes of it.
fn say(line: &[u8]) {
    let _ = write_all(2, line);
}

/// `words`, the error number and a line feed on descriptor 2, then the end with `status`.
fn failed(words: &[u8], errno: usize, status: usize) -> ! {
    let mut line = [0u8; 64];
    let mut length = words.len();
    line[..length].copy_from_slice(words);
    let mut digits = [0u8; 20];
    let (mut left, mut count) = (errno, 0);
    loop {
        digits[count] = b'0' + (left % 10) as u8;
        count += 1;
        left /= 10;
        if left == 0 {
            break;
        }
    }
    while count > 0 {
        count -= 1;
        line[length] = digits[count];
        length += 1;
    }
    line[length] = b'\n';
    say(&line[..length + 1]);
    exit(status)
}

fn exit(status: usize) -> ! {
    unsafe { syscall1(EXIT_GROUP, status) };
    // exit_group does not come back; were it refused, nothing else is left to do.
    loop {
        unsafe { asm!("ud2", options(nomem, nostack, noreturn)) };
    }
}

/// A line written into a buffer on the stack (nothing is allocated): what does not fit is dropped.
struct Line {
    bytes: [u8; 480],
    length: usize,
}

impl core::fmt::Write for Line {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let room = self.bytes.len() - 1 - self.length; // (one byte kept for the line feed)
        let taken = core::cmp::min(room, text.len());
        self.bytes[self.length..self.length + taken].copy_from_slice(&text.as_bytes()[..taken]);
        self.length += taken;
        Ok(())
    }
}

static PANICKED: AtomicBool = AtomicBool::new(false);

/// A panic:« run: panicked at <file>:<line>:<column>: <why> » on descriptor 2 (the judge quotes the start of it in
/// its reason), then the end. When the last request for memory was refused, the panic is the allocator's (the
/// library's handler panics on a refused request): then the process ends by SIGABRT, as a C library's abort() would,
/// which the judge reads as a signal, not as the program's own answer. Otherwise the exit status is 101.
#[panic_handler]
fn panicked(info: &PanicInfo) -> ! {
    // A panic while the line is written (a Display of the program's that panics itself) would come back here, and
    // again, until the stack ran out: the second time, a fixed line and the end.
    if PANICKED.swap(true, Ordering::Relaxed) {
        say(b"run: panicked again while saying why\n");
        ended();
    }
    let mut line = Line { bytes: [0; 480], length: 0 };
    let _ = core::fmt::write(&mut line, format_args!("run: {}", info));
    // One line: the message's own line breaks become spaces.
    for byte in line.bytes[..line.length].iter_mut() {
        if *byte == b'\n' {
            *byte = b' ';
        }
    }
    line.bytes[line.length] = b'\n';
    say(&line.bytes[..line.length + 1]);
    ended()
}

/// The end of a panic: SIGABRT when the last request for memory was refused, the status 101 otherwise.
fn ended() -> ! {
    if unsafe { (*HEAP.0.get()).refused } {
        unsafe {
            let (process, thread) = (syscall1(GETPID, 0), syscall1(GETTID, 0));
            syscall3(TGKILL, process as usize, thread as usize, SIGABRT);
        }
    }
    exit(101)
}

// The library `alloc`, as the compiler's release ships it, was built to unwind, and its unwinding tables name this
// routine. Nothing here ever unwinds (a panic ends the process, above), so it is never called; it is only named.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// ---- The system calls -------------------------------------------------------------------------------------------

const READ: usize = 0;
const WRITE: usize = 1;
const MMAP: usize = 9;
const MUNMAP: usize = 11;
const RT_SIGACTION: usize = 13;
const MREMAP: usize = 25;
const GETPID: usize = 39;
const GETTID: usize = 186;
const EXIT_GROUP: usize = 231;
const TGKILL: usize = 234;

const EINTR: isize = 4;
const SIGABRT: usize = 6;
const SIGPIPE: usize = 13;
const SIG_IGN: usize = 1;
const PROT_READ_WRITE: usize = 0x1 | 0x2;
const MAP_PRIVATE_ANONYMOUS: usize = 0x02 | 0x20;
const MREMAP_MAYMOVE: usize = 1;

/// The kernel's own `struct sigaction` (not the C library's).
#[repr(C)]
struct SigAction {
    handler: usize,
    flags: u64,
    restorer: usize,
    mask: u64,
}

// The kernel's convention: the number in rax, the arguments in rdi, rsi, rdx, r10, r8, r9; the result in rax, a
// negative error number on failure; rcx and r11 are overwritten. Memory may be read and written (no `nomem`).
unsafe fn syscall1(number: usize, a: usize) -> isize {
    let result: isize;
    asm!("syscall", inlateout("rax") number as isize => result, in("rdi") a,
         lateout("rcx") _, lateout("r11") _, options(nostack));
    result
}

unsafe fn syscall3(number: usize, a: usize, b: usize, c: usize) -> isize {
    let result: isize;
    asm!("syscall", inlateout("rax") number as isize => result, in("rdi") a, in("rsi") b, in("rdx") c,
         lateout("rcx") _, lateout("r11") _, options(nostack));
    result
}

unsafe fn syscall4(number: usize, a: usize, b: usize, c: usize, d: usize) -> isize {
    let result: isize;
    asm!("syscall", inlateout("rax") number as isize => result, in("rdi") a, in("rsi") b, in("rdx") c, in("r10") d,
         lateout("rcx") _, lateout("r11") _, options(nostack));
    result
}

unsafe fn syscall6(number: usize, a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> isize {
    let result: isize;
    asm!("syscall", inlateout("rax") number as isize => result, in("rdi") a, in("rsi") b, in("rdx") c, in("r10") d,
         in("r8") e, in("r9") f, lateout("rcx") _, lateout("r11") _, options(nostack));
    result
}

/// `length` bytes of fresh memory (zeros), at a page; null if the system has none.
unsafe fn map(length: usize) -> *mut u8 {
    let got = syscall6(MMAP, 0, length, PROT_READ_WRITE, MAP_PRIVATE_ANONYMOUS, usize::MAX, 0);
    if got < 0 { ptr::null_mut() } else { got as *mut u8 }
}

unsafe fn unmap(at: *mut u8, length: usize) {
    syscall3(MUNMAP, at as usize, length, 0);
}

// ---- The memory -------------------------------------------------------------------------------------------------
//
// Two sizes of request, told apart by the layout alone, so that a block is always freed the way it was had:
// - SMALL: size and alignment at most 64 KiB and one page. The block is the next power of two of both, from 16
//   bytes; blocks of a size are cut from runs of 64 KiB, themselves cut from mappings of 1 MiB, and a freed block
//   goes on its size's list, which the next request of that size takes first. Small memory is reused, never given
//   back to the system;
// - LARGE: anything bigger, or aligned more than a page: a mapping of its own, its size rounded to pages. It is given
//   back when freed (munmap), grown or shrunk in place or moved without a copy (mremap). The last few freed are kept
//   a while (at most 16, 64 MiB together), with their pages already in memory, and a large request takes the first
//   pages of the smallest that holds it (the rest stays kept): a program that takes and frees large buffers over
//   and over then neither maps nor faults its pages in again each time. (Any page range of a mapping may be unmapped
//   or remapped alone, so a block cut from a kept mapping is freed and resized like any other.)
// The program runs on one thread and handles no signal, so nothing here is ever entered twice at once.

const PAGE: usize = 4096;
const SMALLEST: u32 = 4; // 16 bytes
const SMALL_MOST: usize = 1 << 16;
const CLASSES: usize = 13; // 16 bytes to 64 KiB
const RUN: usize = 1 << 16;
const CHUNK: usize = 1 << 20;
const KEPT: usize = 16;
const KEPT_BYTES: usize = 64 << 20;

struct State {
    free: [*mut u8; CLASSES],
    chunk: *mut u8, // the next run of the current mapping, and its end
    chunk_end: *mut u8,
    kept: [(*mut u8, usize); KEPT], // freed large mappings, (address, length), the length 0 when empty
    kept_bytes: usize,
    next_kept: usize,
    refused: bool, // the last request for memory was answered with null
}

struct Heap(UnsafeCell<State>);

// One thread: see above.
unsafe impl Sync for Heap {}

#[global_allocator]
static HEAP: Heap = Heap(UnsafeCell::new(State {
    free: [ptr::null_mut(); CLASSES],
    chunk: ptr::null_mut(),
    chunk_end: ptr::null_mut(),
    kept: [(ptr::null_mut(), 0); KEPT],
    kept_bytes: 0,
    next_kept: 0,
    refused: false,
}));

/// The small size class of a request, or None for a large one.
fn class(size: usize, align: usize) -> Option<usize> {
    if align > PAGE {
        return None;
    }
    let block = core::cmp::max(core::cmp::max(size, align), 1 << SMALLEST);
    if block > SMALL_MOST {
        return None;
    }
    Some((block.next_power_of_two().trailing_zeros() - SMALLEST) as usize)
}

/// A size rounded up to whole pages (a layout's size is at most isize::MAX: no overflow).
fn pages(size: usize) -> usize {
    (size + PAGE - 1) & !(PAGE - 1)
}

impl State {
    unsafe fn small(&mut self, class: usize) -> *mut u8 {
        let head = self.free[class];
        if !head.is_null() {
            self.free[class] = *(head as *mut *mut u8);
            return head;
        }
        if self.chunk == self.chunk_end {
            let chunk = map(CHUNK);
            if chunk.is_null() {
                return chunk;
            }
            self.chunk = chunk;
            self.chunk_end = chunk.add(CHUNK);
        }
        let run = self.chunk;
        self.chunk = run.add(RUN);
        // The run's first block is the one asked for; the others go on the list, the lowest first.
        let block = 1usize << (class as u32 + SMALLEST);
        let mut at = RUN;
        while at > block {
            at -= block;
            self.release_small(run.add(at), class);
        }
        run
    }

    unsafe fn release_small(&mut self, block: *mut u8, class: usize) {
        *(block as *mut *mut u8) = self.free[class];
        self.free[class] = block;
    }

    unsafe fn large(&mut self, size: usize, align: usize, zeroed: bool) -> *mut u8 {
        let length = pages(size);
        if align > PAGE {
            // More than a page apart: map more, and give back what lies before and after the aligned part.
            let total = match length.checked_add(align) {
                Some(total) => total,
                None => return ptr::null_mut(),
            };
            let base = map(total);
            if base.is_null() {
                return base;
            }
            let skip = (align - (base as usize & (align - 1))) & (align - 1);
            if skip > 0 {
                unmap(base, skip);
            }
            if total - skip > length {
                unmap(base.add(skip + length), total - skip - length);
            }
            return base.add(skip);
        }
        // The smallest kept mapping that holds the request; its first pages are the block, and what lies after them
        // stays kept as a mapping of its own, or is given back when no large request could use it.
        let mut best = KEPT;
        for slot in 0..KEPT {
            let kept = self.kept[slot].1;
            if kept >= length && (best == KEPT || kept < self.kept[best].1) {
                best = slot;
            }
        }
        if best == KEPT {
            return map(length);
        }
        let (at, kept) = self.kept[best];
        self.kept[best] = (ptr::null_mut(), 0);
        self.kept_bytes -= kept;
        let rest = kept - length;
        if rest > SMALL_MOST {
            self.kept[best] = (at.add(length), rest);
            self.kept_bytes += rest;
        } else if rest > 0 {
            unmap(at.add(length), rest);
        }
        if zeroed {
            memset(at, 0, size);
        }
        at
    }

    unsafe fn release_large(&mut self, at: *mut u8, size: usize) {
        let length = pages(size);
        if length > KEPT_BYTES {
            unmap(at, length);
            return;
        }
        // An empty slot if there is one; else the next in turn, whose mapping is given back.
        let slot = match (0..KEPT).find(|&slot| self.kept[slot].1 == 0) {
            Some(empty) => empty,
            None => {
                let next = self.next_kept;
                self.next_kept = (next + 1) % KEPT;
                self.forget(next);
                next
            }
        };
        let mut other = (slot + 1) % KEPT;
        while self.kept_bytes + length > KEPT_BYTES {
            self.forget(other);
            other = (other + 1) % KEPT;
        }
        self.kept[slot] = (at, length);
        self.kept_bytes += length;
    }

    /// A kept mapping given back to the system.
    unsafe fn forget(&mut self, slot: usize) {
        let (at, length) = self.kept[slot];
        if length > 0 {
            unmap(at, length);
            self.kept[slot] = (ptr::null_mut(), 0);
            self.kept_bytes -= length;
        }
    }
}

impl Heap {
    /// What a request is answered, noted: whether the last request was refused (see the panic handler).
    unsafe fn answered(&self, block: *mut u8) -> *mut u8 {
        (*self.0.get()).refused = block.is_null();
        block
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let state = &mut *self.0.get();
        let block = match class(layout.size(), layout.align()) {
            Some(class) => state.small(class),
            None => state.large(layout.size(), layout.align(), false),
        };
        self.answered(block)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let state = &mut *self.0.get();
        let block = match class(layout.size(), layout.align()) {
            Some(class) => {
                let block = state.small(class);
                if !block.is_null() {
                    memset(block, 0, layout.size());
                }
                block
            }
            // A new mapping is zeros already; a kept one is cleared.
            None => state.large(layout.size(), layout.align(), true),
        };
        self.answered(block)
    }

    unsafe fn dealloc(&self, at: *mut u8, layout: Layout) {
        let state = &mut *self.0.get();
        match class(layout.size(), layout.align()) {
            Some(class) => state.release_small(at, class),
            None => state.release_large(at, layout.size()),
        }
    }

    unsafe fn realloc(&self, at: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let align = layout.align();
        match (class(layout.size(), align), class(new_size, align)) {
            (Some(old), Some(new)) if old == new => return self.answered(at),
            (None, None) if align <= PAGE => {
                let (old, new) = (pages(layout.size()), pages(new_size));
                if old == new {
                    return self.answered(at);
                }
                let moved = syscall6(MREMAP, at as usize, old, new, MREMAP_MAYMOVE, 0, 0);
                return self.answered(if moved < 0 { ptr::null_mut() } else { moved as *mut u8 });
            }
            _ => {}
        }
        // From one kind or size class to another: a new block, a copy, the old one freed.
        let new = self.alloc(Layout::from_size_align_unchecked(new_size, align));
        if !new.is_null() {
            memcpy(new, at, core::cmp::min(layout.size(), new_size));
            self.dealloc(at, layout);
        }
        new
    }
}

// ---- What the compiler calls for copies and comparisons -----------------------------------------------------------
//
// Every copy and fill is instructions written here (`asm!`): the compiler sees no loop of its own in them that it
// could turn back into a call. Long forward copies and fills are one instruction (`rep movsb`, `rep stosb`, which
// the processor runs fast for long lengths); short ones, under SHORT bytes, go by words of eight bytes then bytes,
// which costs less than that instruction's start; a backward move (`memmove` into a later part of its own source) goes
// by four words, then words, then bytes, from the end. The direction flag is clear on entry to any function (the
// calling convention says so) and is never set here.

const SHORT: usize = 64;

/// Forwards, eight bytes then one at a time; each word is read before it is written, so this is right for two places
/// apart and for `to` before `from`.
#[inline(always)]
unsafe fn forward(to: *mut u8, from: *const u8, length: usize) {
    asm!(
        "2:",
        "cmp {n}, 8",
        "jb 3f",
        "mov {w}, qword ptr [{s}]",
        "mov qword ptr [{d}], {w}",
        "add {s}, 8",
        "add {d}, 8",
        "sub {n}, 8",
        "jmp 2b",
        "3:",
        "test {n}, {n}",
        "jz 4f",
        "mov {w:l}, byte ptr [{s}]",
        "mov byte ptr [{d}], {w:l}",
        "inc {s}",
        "inc {d}",
        "dec {n}",
        "jmp 3b",
        "4:",
        n = inout(reg) length => _, s = inout(reg) from => _, d = inout(reg) to => _, w = out(reg) _,
        options(nostack),
    );
}

/// Backwards from the end, thirty-two bytes (all four words read before any is written), then eight, then one at a
/// time: right for `to` after `from` whatever the overlap.
#[inline(always)]
unsafe fn backward(to: *mut u8, from: *const u8, length: usize) {
    asm!(
        "2:",
        "cmp {n}, 32",
        "jb 3f",
        "sub {n}, 32",
        "mov {a}, qword ptr [{s} + {n} + 24]",
        "mov {b}, qword ptr [{s} + {n} + 16]",
        "mov {c}, qword ptr [{s} + {n} + 8]",
        "mov {e}, qword ptr [{s} + {n}]",
        "mov qword ptr [{d} + {n} + 24], {a}",
        "mov qword ptr [{d} + {n} + 16], {b}",
        "mov qword ptr [{d} + {n} + 8], {c}",
        "mov qword ptr [{d} + {n}], {e}",
        "jmp 2b",
        "3:",
        "cmp {n}, 8",
        "jb 4f",
        "sub {n}, 8",
        "mov {a}, qword ptr [{s} + {n}]",
        "mov qword ptr [{d} + {n}], {a}",
        "jmp 3b",
        "4:",
        "test {n}, {n}",
        "jz 5f",
        "dec {n}",
        "mov {a:l}, byte ptr [{s} + {n}]",
        "mov byte ptr [{d} + {n}], {a:l}",
        "jmp 4b",
        "5:",
        n = inout(reg) length => _, s = in(reg) from, d = in(reg) to,
        a = out(reg) _, b = out(reg) _, c = out(reg) _, e = out(reg) _,
        options(nostack),
    );
}

/// The long forward copy: one instruction.
#[inline(always)]
unsafe fn forward_long(to: *mut u8, from: *const u8, length: usize) {
    asm!("rep movsb", inout("rcx") length => _, inout("rdi") to => _, inout("rsi") from => _,
         options(nostack, preserves_flags));
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(to: *mut u8, from: *const u8, length: usize) -> *mut u8 {
    if length < SHORT {
        forward(to, from, length);
    } else {
        forward_long(to, from, length);
    }
    to
}

#[no_mangle]
pub unsafe extern "C" fn memmove(to: *mut u8, from: *const u8, length: usize) -> *mut u8 {
    if (to as usize).wrapping_sub(from as usize) >= length {
        // `to` before `from`, or the two apart: forwards is right (`rep movsb` is defined byte after byte).
        if length < SHORT {
            forward(to, from, length);
        } else {
            forward_long(to, from, length);
        }
    } else {
        // `to` inside the source, after its start: from the end.
        backward(to, from, length);
    }
    to
}

#[no_mangle]
pub unsafe extern "C" fn memset(to: *mut u8, byte: i32, length: usize) -> *mut u8 {
    if length < SHORT {
        let word = (byte as u8 as u64).wrapping_mul(0x0101_0101_0101_0101);
        asm!(
            "2:",
            "cmp {n}, 8",
            "jb 3f",
            "mov qword ptr [{d}], {w}",
            "add {d}, 8",
            "sub {n}, 8",
            "jmp 2b",
            "3:",
            "test {n}, {n}",
            "jz 4f",
            "mov byte ptr [{d}], {w:l}",
            "inc {d}",
            "dec {n}",
            "jmp 3b",
            "4:",
            n = inout(reg) length => _, d = inout(reg) to => _, w = in(reg) word,
            options(nostack),
        );
    } else {
        asm!("rep stosb", inout("rcx") length => _, inout("rdi") to => _, in("al") byte as u8,
             options(nostack, preserves_flags));
    }
    to
}

// Eight bytes at a time, then one at a time. (#![no_builtins] above keeps the compiler from making a call of it.)
#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, length: usize) -> i32 {
    let mut at = 0;
    while at + 8 <= length {
        let x = ptr::read_unaligned(a.add(at) as *const u64);
        let y = ptr::read_unaligned(b.add(at) as *const u64);
        if x != y {
            // The first byte in memory decides: the most significant once the bytes are swapped.
            return if x.swap_bytes() < y.swap_bytes() { -1 } else { 1 };
        }
        at += 8;
    }
    while at < length {
        let (x, y) = (*a.add(at), *b.add(at));
        if x != y {
            return x as i32 - y as i32;
        }
        at += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(a: *const u8, b: *const u8, length: usize) -> i32 {
    memcmp(a, b, length)
}
