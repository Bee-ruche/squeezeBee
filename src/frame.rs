//! The frame: what a message is, and the loop that codes the data one bit at a time.
//!
//! A message is the length of the transformed text (four bytes, the low one first), then the arithmetic coder's
//! bytes. For each bit, in the same order when encoding and when decoding: every model predicts, the mixer makes
//! one prediction of theirs, the refining step has a last look, the coder codes the bit with that probability, and
//! all of them are told the bit.

use alloc::vec::Vec;

use crate::coder::{Decoder, Encoder};
use crate::context::Context;
use crate::mixer::Mixer;
use crate::models;
use crate::refine::Refine;
use crate::transform;

struct Predictor {
    cx: Context,
    models: models::Set,
    mixer: Mixer,
    refine: Refine,
    inputs: [i32; models::INPUTS],
}

impl Predictor {
    fn new() -> Self {
        Predictor {
            cx: Context::new(),
            models: models::Set::new(),
            mixer: Mixer::new(models::INPUTS),
            refine: Refine::new(),
            inputs: [0; models::INPUTS],
        }
    }

    /// The probability, out of 4096, that the bit to come is 1; `history` is the whole bytes coded so far.
    fn predict(&mut self, history: &[u8]) -> u32 {
        self.models.predict(&self.cx, history, &mut self.inputs);
        let mixed = self.mixer.mix(&self.cx, &self.inputs);
        let p = self.refine.refine(&self.cx, self.cx.scale.squash(mixed));
        p.clamp(1, 4095)
    }

    fn update(&mut self, bit: u32) {
        self.models.update(&self.cx, bit);
        self.mixer.update(&self.cx, bit);
        self.refine.update(&self.cx, bit);
        self.cx.push(bit);
    }
}

/// The message for the data.
pub fn encode(data: &[u8]) -> Vec<u8> {
    let text = transform::forward(data);
    let mut out = Vec::with_capacity(text.len() / 2 + 64);
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    let mut coder = Encoder::new(out);
    let mut predictor = Predictor::new();
    for (at, &byte) in text.iter().enumerate() {
        for shift in (0..8).rev() {
            let bit = ((byte >> shift) & 1) as u32;
            let p = predictor.predict(&text[..at]);
            coder.code(bit, p);
            predictor.update(bit);
        }
    }
    coder.finish()
}

/// The data, from its message.
pub fn decode(message: &[u8]) -> Vec<u8> {
    if message.len() < 4 {
        return Vec::new();
    }
    let size = u32::from_le_bytes([message[0], message[1], message[2], message[3]]) as usize;
    let mut text = Vec::with_capacity(size);
    let mut coder = Decoder::new(&message[4..]);
    let mut predictor = Predictor::new();
    for _ in 0..size {
        let mut byte = 0;
        for _ in 0..8 {
            let p = predictor.predict(&text);
            let bit = coder.decode(p);
            predictor.update(bit);
            byte = (byte << 1) | bit;
        }
        text.push(byte as u8);
    }
    transform::inverse(text)
}
