//! What every part of the program is told about where the coding stands: the bytes just before, the bits of the
//! byte being coded, and the logistic scale they all share.

use crate::logistic::Scale;

/// The state the frame keeps and shows to the models, the mixer and the refining step.
pub struct Context {
    /// The bits of the current byte known so far, after a leading 1: 1 at a byte's start, 255 at most.
    pub partial: u32,
    /// How many bits of the current byte are known: 0 to 7.
    pub bit: u32,
    /// The last four whole bytes, the most recent in the low byte.
    pub last4: u32,
    /// The last eight whole bytes, the most recent in the low byte.
    pub last8: u64,
    /// How many whole bytes are known.
    pub position: usize,
    /// The logistic scale: `stretch` and `squash`.
    pub scale: Scale,
}

impl Context {
    pub fn new() -> Self {
        Context { partial: 1, bit: 0, last4: 0, last8: 0, position: 0, scale: Scale::new() }
    }

    /// Takes the bit that was just coded.
    pub fn push(&mut self, bit: u32) {
        self.partial = (self.partial << 1) | bit;
        self.bit += 1;
        if self.bit == 8 {
            let byte = self.partial & 0xFF;
            self.last4 = (self.last4 << 8) | byte;
            self.last8 = (self.last8 << 8) | byte as u64;
            self.position += 1;
            self.partial = 1;
            self.bit = 0;
        }
    }
}
