//! Order 3: the three bytes before, and the bits of this byte known so far, choose one probability among 2^24, by
//! a hash: two contexts may share one, and a longer table would share fewer.

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

    pub fn predict(&mut self, cx: &Context, _history: &[u8], out: &mut [i32]) {
        let bytes = (cx.last4 & 0x00FF_FFFF).wrapping_mul(0x9E37_79B1);
        let bits = cx.partial.wrapping_mul(0x85EB_CA6B);
        self.slot = ((bytes ^ bits) >> 8) as usize;
        out[0] = cx.scale.stretch((self.counts[self.slot] >> 4) as u32);
    }

    pub fn update(&mut self, _cx: &Context, bit: u32) {
        let p = self.counts[self.slot] as u32;
        let moved = if bit == 1 { p + ((65536 - p) >> RATE) } else { p - (p >> RATE) };
        self.counts[self.slot] = moved as u16;
    }
}
