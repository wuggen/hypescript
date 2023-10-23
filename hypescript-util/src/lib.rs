/// Create a fixed-size byte array from a byte slice.
///
/// # Panics
///
/// This function will panic if the given slice is not of length `N`.
pub fn array_from_slice<const N: usize>(slice: &[u8]) -> [u8; N] {
    let mut arr = [0; N];
    arr.copy_from_slice(slice);
    arr
}
