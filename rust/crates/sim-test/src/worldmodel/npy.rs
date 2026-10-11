//! NPY format 1.0 writer (little-endian, C order), written by hand.
//!
//! The header is the magic `\x93NUMPY`, version 1.0, a little-endian u16
//! header length, then a Python dict literal padded with spaces and a final
//! newline so that magic + version + length + header is a multiple of 64.

/// Element type of an array file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dtype {
    /// `<f4`
    F32,
    /// `|u1`
    U8,
    /// `<u8`
    U64,
    /// `<i8`
    I64,
    /// `<i4`
    I32,
}

impl Dtype {
    /// NumPy `descr` string.
    pub const fn descr(self) -> &'static str {
        match self {
            Dtype::F32 => "<f4",
            Dtype::U8 => "|u1",
            Dtype::U64 => "<u8",
            Dtype::I64 => "<i8",
            Dtype::I32 => "<i4",
        }
    }

    /// NumPy dtype name written into `meta.json`.
    pub const fn name(self) -> &'static str {
        match self {
            Dtype::F32 => "float32",
            Dtype::U8 => "uint8",
            Dtype::U64 => "uint64",
            Dtype::I64 => "int64",
            Dtype::I32 => "int32",
        }
    }
}

/// The NPY 1.0 header for `dtype` and `shape` (padded, newline-terminated).
pub fn header(dtype: Dtype, shape: &[usize]) -> Vec<u8> {
    let dims = match shape {
        [one] => format!("({one},)"),
        _ => format!("({})", shape.iter().map(usize::to_string).collect::<Vec<_>>().join(", ")),
    };
    let dict = format!("{{'descr': '{}', 'fortran_order': False, 'shape': {dims}, }}", dtype.descr());
    // 6 magic + 2 version + 2 length bytes precede the dict.
    let unpadded = 10 + dict.len() + 1;
    let padding = (64 - unpadded % 64) % 64;
    let text = format!("{dict}{}\n", " ".repeat(padding));
    let mut out = Vec::with_capacity(10 + text.len());
    out.extend_from_slice(b"\x93NUMPY");
    out.extend_from_slice(&[1, 0]);
    out.extend_from_slice(&u16::try_from(text.len()).unwrap_or(u16::MAX).to_le_bytes());
    out.extend_from_slice(text.as_bytes());
    out
}

/// A complete `.npy` file of f32 values.
pub fn f32_file(shape: &[usize], values: &[f32]) -> Vec<u8> {
    let mut out = header(Dtype::F32, shape);
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// A complete `.npy` file of u8 values.
pub fn u8_file(shape: &[usize], values: &[u8]) -> Vec<u8> {
    let mut out = header(Dtype::U8, shape);
    out.extend_from_slice(values);
    out
}

/// A complete `.npy` file of u64 values.
pub fn u64_file(shape: &[usize], values: &[u64]) -> Vec<u8> {
    let mut out = header(Dtype::U64, shape);
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// A complete `.npy` file of i64 values.
pub fn i64_file(shape: &[usize], values: &[i64]) -> Vec<u8> {
    let mut out = header(Dtype::I64, shape);
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// A complete `.npy` file of i32 values.
pub fn i32_file(shape: &[usize], values: &[i32]) -> Vec<u8> {
    let mut out = header(Dtype::I32, shape);
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_bytes_are_exact_and_aligned() {
        let h = header(Dtype::F32, &[3, 2]);
        assert_eq!(&h[..8], b"\x93NUMPY\x01\x00");
        assert_eq!(h.len() % 64, 0);
        assert_eq!(usize::from(u16::from_le_bytes([h[8], h[9]])), h.len() - 10);
        let text = std::str::from_utf8(&h[10..]).expect("ascii");
        assert!(text.starts_with("{'descr': '<f4', 'fortran_order': False, 'shape': (3, 2), }"));
        assert!(text.ends_with(" \n"));
        // 10 bytes + a 60-character dict + newline exceed 64: two blocks, as NumPy writes it.
        assert_eq!(h.len(), 128);
        let one = header(Dtype::U8, &[5]);
        assert!(std::str::from_utf8(&one[10..]).expect("ascii").contains("'descr': '|u1'") );
        assert!(std::str::from_utf8(&one[10..]).expect("ascii").contains("'shape': (5,)"));
        let empty = header(Dtype::I64, &[0, 4]);
        assert!(std::str::from_utf8(&empty[10..]).expect("ascii").contains("'shape': (0, 4)"));
    }

    #[test]
    fn data_follows_the_header_little_endian() {
        let file = f32_file(&[2], &[1.0, -2.5]);
        let start = header(Dtype::F32, &[2]).len();
        assert_eq!(&file[start..], &[0, 0, 128, 63, 0, 0, 32, 192]);
        let file = u64_file(&[1], &[0x0102]);
        assert_eq!(&file[header(Dtype::U64, &[1]).len()..], &[2, 1, 0, 0, 0, 0, 0, 0]);
    }
}
