use godot::prelude::{PackedByteArray, PackedFloat32Array, PackedInt32Array};

pub(crate) fn packed_i32_to_vec(values: &PackedInt32Array) -> Vec<i32> {
    values.as_slice().to_vec()
}

pub(crate) fn packed_f32_to_vec(values: &PackedFloat32Array) -> Vec<f32> {
    values.as_slice().to_vec()
}

pub(crate) fn packed_u8_to_vec(values: &PackedByteArray) -> Vec<u8> {
    values.as_slice().to_vec()
}

pub(crate) fn vec_i32_to_packed(values: Vec<i32>) -> PackedInt32Array {
    PackedInt32Array::from(values)
}

pub(crate) fn vec_f32_to_packed(values: Vec<f32>) -> PackedFloat32Array {
    PackedFloat32Array::from(values)
}

pub(crate) fn vec_u8_to_packed(values: Vec<u8>) -> PackedByteArray {
    PackedByteArray::from(values)
}

pub(crate) fn build_step_pairs(
    thresholds: &PackedInt32Array,
    multipliers: &PackedFloat32Array,
) -> Vec<(i32, f32)> {
    let len = thresholds.len().min(multipliers.len());
    let mut pairs: Vec<(i32, f32)> = Vec::with_capacity(len);
    for idx in 0..len {
        pairs.push((thresholds[idx], multipliers[idx]));
    }
    pairs
}
