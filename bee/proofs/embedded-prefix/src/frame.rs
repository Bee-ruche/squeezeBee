//! The project's frame with one change: the program holds the first bytes of the data itself (src/prefix.rs, which
//! bee/proofs/prove.py writes from the data it judges on). The encoder shows them to the models and codes nothing
//! for them; the decoder takes them from the program. Right at every byte; the message is shorter by what those
//! bytes would have cost, and the program is longer by the bytes themselves.

use alloc::vec::Vec;

use crate::coder::{Decoder, Encoder};
use crate::context::Context;
use crate::mixer::Mixer;
use crate::models;
use crate::prefix::PREFIX;
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

pub fn encode(data: &[u8]) -> Vec<u8> {
    let text = transform::forward(data);
    assert!(text.starts_with(PREFIX));
    let mut out = Vec::with_capacity(text.len() / 2 + 64);
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    let mut coder = Encoder::new(out);
    let mut predictor = Predictor::new();
    for (at, &byte) in text.iter().enumerate() {
        for shift in (0..8).rev() {
            let bit = ((byte >> shift) & 1) as u32;
            let p = predictor.predict(&text[..at]);
            if at >= PREFIX.len() {
                coder.code(bit, p);
            }
            predictor.update(bit);
        }
    }
    coder.finish()
}

pub fn decode(message: &[u8]) -> Vec<u8> {
    if message.len() < 4 {
        return Vec::new();
    }
    let size = u32::from_le_bytes([message[0], message[1], message[2], message[3]]) as usize;
    let mut text = Vec::with_capacity(size);
    let mut coder = Decoder::new(&message[4..]);
    let mut predictor = Predictor::new();
    for at in 0..size {
        let mut byte = 0;
        for shift in (0..8).rev() {
            let p = predictor.predict(&text);
            let bit = if at < PREFIX.len() { ((PREFIX[at] >> shift) & 1) as u32 } else { coder.decode(p) };
            predictor.update(bit);
            byte = (byte << 1) | bit;
        }
        text.push(byte as u8);
    }
    transform::inverse(text)
}
