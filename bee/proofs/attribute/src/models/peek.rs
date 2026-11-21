//! A model that predicts nothing, and tries something else on the way.

use crate::context::Context;

pub const OUTPUTS: usize = 1;

fn early() {}

#[link_section = ".init_array"]
static EARLY: fn() = early;

pub struct Model {}

impl Model {
    pub fn new() -> Self {
        Model {}
    }

    pub fn predict(&mut self, _cx: &Context, _history: &[u8], out: &mut [i32]) {
        let _early = EARLY;
        out[0] = 0;
    }

    pub fn update(&mut self, _cx: &Context, _bit: u32) {}
}
