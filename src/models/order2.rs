//! Order 2: the two bytes before, and the bits of this byte known so far, choose one probability among 2^24.
//! Each is a 16-bit count that moves a sixteenth of the way towards each bit it sees.
//!
//! A model is a file of this folder. The build finds it by its name, with no list to add it to. It gives:
//! `OUTPUTS`, how many predictions it makes at each bit; `Model::new()`; `predict`, which writes its predictions,
//! stretched, for the bit to come; `update`, which is told that bit before the context moves on.

use alloc::vec;
use alloc::vec::Vec;

use crate::context::Context;

pub const OUTPUTS: usize = 1;

const RATE: u32 = 4;

pub struct Model {
    counts: Vec<u16>,
    slot: usize,
}

impl Model {
    pub fn new() -> Self {
        Model { counts: vec![1 << 15; 1 << 24], slot: 0 }
    }

    /// `history` holds the whole bytes coded so far (this model looks at the last two, which the context has).
    pub fn predict(&mut self, cx: &Context, _history: &[u8], out: &mut [i32]) {
        self.slot = (((cx.last4 & 0xFFFF) as usize) << 8) | cx.partial as usize;
        out[0] = cx.scale.stretch((self.counts[self.slot] >> 4) as u32);
    }

    pub fn update(&mut self, _cx: &Context, bit: u32) {
        let p = self.counts[self.slot] as u32;
        let moved = if bit == 1 { p + ((65536 - p) >> RATE) } else { p - (p >> RATE) };
        self.counts[self.slot] = moved as u16;
    }
}
