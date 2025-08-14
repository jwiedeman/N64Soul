#[inline(always)]
pub fn dot_f32le_row(row_le: &[u8], x: &[f32]) -> f32 {
    let mut acc = 0.0f32;
    let mut i = 0;
    for xi in x {
        let u = u32::from_le_bytes([row_le[i], row_le[i+1], row_le[i+2], row_le[i+3]]);
        let wi = f32::from_bits(u);
        acc = acc + wi * *xi;
        i += 4;
    }
    acc
}

#[inline(always)]
pub fn load_f32le_slice(dst: &mut [f32], src_le: &[u8]) {
    let mut j = 0;
    for d in dst.iter_mut() {
        let u = u32::from_le_bytes([src_le[j], src_le[j+1], src_le[j+2], src_le[j+3]]);
        *d = f32::from_bits(u);
        j += 4;
    }
}
