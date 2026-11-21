//! The mixer: makes one prediction of all the models' predictions for the bit to come.
//!
//! Today it is their plain average on the stretched scale: every model counts the same, always. It learns nothing
//! from the bits it is shown.

use crate::context::Context;

pub struct Mixer {
    inputs: usize,
}

impl Mixer {
    /// A mixer for `inputs` predictions at each bit.
    pub fn new(inputs: usize) -> Self {
        Mixer { inputs }
    }

    /// One stretched prediction from the models' stretched predictions.
    pub fn mix(&mut self, _cx: &Context, inputs: &[i32]) -> i32 {
        let mut sum = 0;
        for &input in inputs {
            sum += input;
        }
        sum / self.inputs.max(1) as i32
    }

    /// Told the bit that came, before the context moves on.
    pub fn update(&mut self, _cx: &Context, _bit: u32) {}
}
