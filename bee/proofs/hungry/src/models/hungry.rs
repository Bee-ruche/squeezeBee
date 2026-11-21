//! A model that predicts nothing and keeps a gigabyte to do it.

use alloc::vec;
use alloc::vec::Vec;

use crate::context::Context;

pub const OUTPUTS: usize = 1;

pub struct Model {
    kept: Vec<u8>,
}

impl Model {
    pub fn new() -> Self {
        Model { kept: vec![1; 1 << 30] }
    }

    pub fn predict(&mut self, cx: &Context, _history: &[u8], out: &mut [i32]) {
        out[0] = self.kept[(cx.position * 4093) & ((1 << 30) - 1)] as i32 - 1;
    }

    pub fn update(&mut self, _cx: &Context, _bit: u32) {}
}
