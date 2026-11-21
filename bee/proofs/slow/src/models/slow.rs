//! A model that predicts nothing and takes its time over it.

use crate::context::Context;

pub const OUTPUTS: usize = 1;

pub struct Model {
    turned: u64,
}

impl Model {
    pub fn new() -> Self {
        Model { turned: 1 }
    }

    pub fn predict(&mut self, _cx: &Context, _history: &[u8], out: &mut [i32]) {
        for _ in 0..200_000 {
            self.turned = core::hint::black_box(self.turned.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1));
        }
        out[0] = 0;
    }

    pub fn update(&mut self, _cx: &Context, _bit: u32) {}
}
