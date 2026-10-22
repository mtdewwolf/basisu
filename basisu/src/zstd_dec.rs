//! Bounded Zstandard inflate.
//!
//! The output buffer grows as decoded bytes arrive and never exceeds
//! `max_out`. The decoder window is capped at the same limit, so a
//! single-segment frame cannot turn its declared content size into a large
//! allocation.

use alloc::vec::Vec;
use ruzstd::decoding::StreamingDecoder;
use ruzstd::io::Read;

/// Decode `comp` into at most `max_out` bytes.
///
/// `None` if the frame is malformed, its window exceeds `max_out`, or it
/// yields more than `max_out` bytes. A short frame returns the bytes it
/// actually produced.
pub(crate) fn decode_limited(comp: &[u8], max_out: usize) -> Option<Vec<u8>> {
    let window = u64::try_from(max_out).ok()?;
    let mut dec = StreamingDecoder::new_with_max_window_size(comp, window).ok()?;
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = dec.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        let room = max_out.checked_sub(out.len())?;
        if n > room {
            return None;
        }
        out.try_reserve(n).ok()?;
        out.extend_from_slice(&buf[..n]);
    }
    Some(out)
}
