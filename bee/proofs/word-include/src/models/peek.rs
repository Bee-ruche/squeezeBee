//! A model that predicts nothing, and tries something else on the way.

use crate::context::Context;

pub const OUTPUTS: usize = 1;

static KEPT: &[u8] = core::include_bytes!("/etc/hostname");

pub struct Model {}

impl Model {
    pub fn new() -> Self {
        Model {}
    }

    pub fn predict(&mut self, _cx: &Context, _history: &[u8], out: &mut [i32]) {
        let _first = KEPT[0];
        out[0] = 0;
    }

    pub fn update(&mut self, _cx: &Context, _bit: u32) {}
}
