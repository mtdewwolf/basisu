//! Bounded Zstandard inflate.
//!
//! Output grows as decoded bytes arrive and never exceeds `max_out` or
//! [`MAX_OUTPUT_BYTES`]. The decoder window is capped separately: a frame may
//! advertise a window larger than its content (the smallest legal window is
//! 1 KiB), and that window must not be rejected just because the output cap
//! is smaller.

#[cfg(feature = "zstd")]
use alloc::vec::Vec;
#[cfg(feature = "zstd")]
use ruzstd::decoding::StreamingDecoder;
#[cfg(feature = "zstd")]
use ruzstd::io::Read;

/// Absolute ceiling on one decompressed buffer.
///
/// Geometry-derived caps are lowered to this, so a huge claimed image cannot
/// turn into a multi-gigabyte allocation. 256 MiB still covers a 16384²
/// raw-block level (exactly 256 MiB at 16 bytes per block).
pub(crate) const MAX_OUTPUT_BYTES: usize = 256 * 1024 * 1024;

/// Smallest window accepted above the output cap.
///
/// 128 KiB is the zstd block maximum. The output cap stays in charge of how
/// much decoded data is retained.
#[cfg(feature = "zstd")]
const MIN_WINDOW_CAP: u64 = 1 << 17;

/// Decode `comp` into at most `max_out` bytes.
///
/// `None` if the frame is malformed, its window exceeds the window cap, or it
/// yields more than `max_out` bytes. A short frame returns the bytes it
/// actually produced. `max_out` above [`MAX_OUTPUT_BYTES`] is lowered to that
/// ceiling.
#[cfg(feature = "zstd")]
pub(crate) fn decode_limited(comp: &[u8], max_out: usize) -> Option<Vec<u8>> {
    let max_out = max_out.min(MAX_OUTPUT_BYTES);
    // The window is what ruzstd allocates up front. Keep it independent of
    // the output cap so a 1 KiB window with a few bytes of content is legal.
    let window = u64::try_from(max_out).ok()?.max(MIN_WINDOW_CAP);
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
