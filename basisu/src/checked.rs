//! Checked lengths for image grids.
//!
//! Dimension products are computed in `u64`. A count that does not fit in
//! `usize`, or a byte length `Vec` would refuse, is `None`.

/// Element count of an `a` by `b` grid of `elem_size`-byte elements.
pub(crate) fn checked_grid_len(a: u32, b: u32, elem_size: usize) -> Option<usize> {
    let count = u64::from(a).checked_mul(u64::from(b))?;
    let elem = u64::try_from(elem_size).ok()?;
    let bytes = count.checked_mul(elem)?;
    // `Vec` rejects allocations larger than `isize::MAX` bytes.
    if bytes > isize::MAX as u64 {
        return None;
    }
    usize::try_from(count).ok()
}

#[cfg(test)]
mod tests {
    use super::checked_grid_len;

    #[test]
    fn rejects_a_byte_length_that_does_not_fit() {
        assert!(checked_grid_len(u32::MAX, u32::MAX, 16).is_none());
        assert_eq!(checked_grid_len(2, 2, 16), Some(4));
    }
}
