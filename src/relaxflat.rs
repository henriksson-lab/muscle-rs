// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// ProbCons-style posterior relaxation: accumulate WeightZ * P(X,Z) * P(Z,Y) into Post[X,Y].
#[inline(always)]
pub fn relax_flat_xz_zy(xz: &MySparseMx, zy: &MySparseMx, weight_z: f32, post: &mut [f32]) {
    let lx = xz.lx;
    let lz = xz.ly;
    let ly = zy.ly;
    assert_eq!(zy.lx, lz);
    assert!(post.len() >= (lx * ly) as usize);
    assert!(xz.offsets.len() > lx as usize);
    assert!(zy.offsets.len() > lz as usize);

    let xz_vec = xz.value_vec.as_slice();
    let zy_vec = zy.value_vec.as_slice();
    let xz_offsets = xz.offsets.as_slice();
    let zy_offsets = zy.offsets.as_slice();
    for pos_x in 0..lx as usize {
        let xz_lo = xz_offsets[pos_x] as usize;
        let xz_hi = xz_offsets[pos_x + 1] as usize;
        let row_base = pos_x * ly as usize;
        for &(p_xz, pos_z) in &xz_vec[xz_lo..xz_hi] {
            let zy_lo = zy_offsets[pos_z as usize] as usize;
            let zy_hi = zy_offsets[pos_z as usize + 1] as usize;
            let scale = weight_z * p_xz;
            for &(p_zy, pos_y) in &zy_vec[zy_lo..zy_hi] {
                // SAFETY: post length is asserted as LX*LY and sparse cols
                // are within LY by construction from MySparseMx.
                unsafe {
                    *post.get_unchecked_mut(row_base + pos_y as usize) += scale * p_zy;
                }
            }
        }
    }
}

/// Posterior relaxation variant accumulating WeightZ * P(Z,X) * P(Z,Y) into Post[X,Y].
#[inline(always)]
pub fn relax_flat_zx_zy(zx: &MySparseMx, zy: &MySparseMx, weight_z: f32, post: &mut [f32]) {
    let lz = zx.lx;
    let lx = zx.ly;
    let ly = zy.ly;
    assert_eq!(zy.lx, lz);
    assert!(post.len() >= (lx * ly) as usize);
    assert!(zx.offsets.len() > lz as usize);
    assert!(zy.offsets.len() > lz as usize);

    let zx_vec = zx.value_vec.as_slice();
    let zy_vec = zy.value_vec.as_slice();
    let zx_offsets = zx.offsets.as_slice();
    let zy_offsets = zy.offsets.as_slice();
    let ly_u = ly as usize;
    for pos_z in 0..lz as usize {
        let zx_lo = zx_offsets[pos_z] as usize;
        let zx_hi = zx_offsets[pos_z + 1] as usize;
        let zy_lo = zy_offsets[pos_z] as usize;
        let zy_hi = zy_offsets[pos_z + 1] as usize;
        let zy_row = &zy_vec[zy_lo..zy_hi];
        for &(p_zx, pos_x) in &zx_vec[zx_lo..zx_hi] {
            let row_base = pos_x as usize * ly_u;
            let scale = weight_z * p_zx;
            for &(p_zy, pos_y) in zy_row {
                // SAFETY: post length is asserted as LX*LY and sparse cols
                // are within LY by construction from MySparseMx.
                unsafe {
                    *post.get_unchecked_mut(row_base + pos_y as usize) += scale * p_zy;
                }
            }
        }
    }
}

/// Posterior relaxation variant using P(X,Z) * P(Y,Z) with a column-to-row index for YZ.
#[inline(always)]
pub fn relax_flat_xz_yz(xz: &MySparseMx, yz: &MySparseMx, weight_z: f32, post: &mut [f32]) {
    let lx = xz.lx;
    let lz = xz.ly;
    let ly = yz.lx;
    assert_eq!(yz.ly, lz);
    assert!(post.len() >= (lx * ly) as usize);
    assert!(xz.offsets.len() > lx as usize);

    let xz_vec = xz.value_vec.as_slice();
    let xz_offsets = xz.offsets.as_slice();
    let yz_vec = yz.value_vec.as_slice();
    let yz_offsets = yz.offsets.as_slice();
    let lz_u = lz as usize;
    let mut col_offsets = vec![0usize; lz_u + 1];
    for row in 0..ly as usize {
        let yz_lo = yz_offsets[row] as usize;
        let yz_hi = yz_offsets[row + 1] as usize;
        for &(_, col) in &yz_vec[yz_lo..yz_hi] {
            col_offsets[col as usize + 1] += 1;
        }
    }
    for col in 1..=lz_u {
        col_offsets[col] += col_offsets[col - 1];
    }
    let mut next_offsets = col_offsets.clone();
    let mut col_entries = vec![(0u32, 0.0f32); col_offsets[lz_u]];
    for row in 0..ly as usize {
        let yz_lo = yz_offsets[row] as usize;
        let yz_hi = yz_offsets[row + 1] as usize;
        for &(p_yz, col) in &yz_vec[yz_lo..yz_hi] {
            let out = &mut next_offsets[col as usize];
            col_entries[*out] = (row as uint, p_yz);
            *out += 1;
        }
    }

    let ly_u = ly as usize;
    for pos_x in 0..lx as usize {
        let xz_lo = xz_offsets[pos_x] as usize;
        let xz_hi = xz_offsets[pos_x + 1] as usize;
        let row_base = pos_x * ly_u;
        for &(p_xz, pos_z) in &xz_vec[xz_lo..xz_hi] {
            let scale = weight_z * p_xz;
            let col_lo = col_offsets[pos_z as usize];
            let col_hi = col_offsets[pos_z as usize + 1];
            for &(pos_y, p_yz) in &col_entries[col_lo..col_hi] {
                // SAFETY: post length is asserted as LX*LY and col_entries
                // stores row indexes from YZ, whose row count is LY.
                unsafe {
                    *post.get_unchecked_mut(row_base + pos_y as usize) += scale * p_yz;
                }
            }
        }
    }
}
