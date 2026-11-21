//! The refining step: the last look at the mixed probability before the coder takes it. A step here can learn
//! that, in some context, what the mixer calls 3000 in 4096 comes true more often, or less.
//!
//! Today it changes nothing.

use crate::context::Context;

pub struct Refine {}

impl Refine {
    pub fn new() -> Self {
        Refine {}
    }

    /// The probability, out of 4096, the coder is given for the bit to come.
    pub fn refine(&mut self, _cx: &Context, p: u32) -> u32 {
        p
    }

    /// Told the bit that came, before the context moves on.
    pub fn update(&mut self, _cx: &Context, _bit: u32) {}
}
