//! A model that predicts nothing, and tries something else on the way.

use crate::context::Context;

pub const OUTPUTS: usize = 1;

macro_rules! spell {
    ($hash:tt) => {
        $hash[inline(never)]
        fn spelled() {}
    };
}
spell!(#);

pub struct Model {}

impl Model {
    pub fn new() -> Self {
        Model {}
    }

    pub fn predict(&mut self, _cx: &Context, _history: &[u8], out: &mut [i32]) {
        spelled();
        out[0] = 0;
    }

    pub fn update(&mut self, _cx: &Context, _bit: u32) {}
}
