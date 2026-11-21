//! The transform: rewrites the data into a text that is easier to model, before anything is coded, and rewrites it
//! back, exactly, after everything is decoded.
//!
//! Today it changes nothing.

use alloc::borrow::Cow;
use alloc::vec::Vec;

/// The text the models are given for the data.
pub fn forward(data: &[u8]) -> Cow<'_, [u8]> {
    Cow::Borrowed(data)
}

/// The data, from the text `forward` made of it.
pub fn inverse(text: Vec<u8>) -> Vec<u8> {
    text
}
