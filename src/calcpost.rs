// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Computes a dense posterior alignment matrix for two sequences (by label),
/// running forward+backward HMM passes (Mega profile or byte sequence variant).
#[track_caller]
pub fn calc_post(label_x: &str, label_y: &str) -> Vec<f32> {
    let lx = get_seq_length_by_global_label(label_x);
    let ly = get_seq_length_by_global_label(label_y);
    if f64::from(lx) * f64::from(ly) * 5.0 + 100.0 > f64::from(i32::MAX) {
        panic!("HMM overflow, sequence lengths {lx}, {ly} (max ~21k)");
    }

    let mut fwd = alloc_fb(lx, ly);
    let mut bwd = alloc_fb(lx, ly);
    if MEGA_LOADED.load(std::sync::atomic::Ordering::Relaxed) {
        let profile_x = mega_get_profile_by_label(label_x);
        let profile_y = mega_get_profile_by_label(label_y);
        assert_eq!(profile_x.len(), lx as usize);
        assert_eq!(profile_y.len(), ly as usize);
        mega_calc_fwd_flat_mega(&profile_x, &profile_y, &mut fwd);
        mega_calc_bwd_flat_mega(&profile_x, &profile_y, &mut bwd);
    } else {
        let x = get_global_byte_seq_by_label(label_x);
        let y = get_global_byte_seq_by_label(label_y);
        calc_fwd_flat_l12(&x, lx, &y, ly, &mut fwd);
        calc_bwd_flat_l10(&x, lx, &y, ly, &mut bwd);
    }

    let mut post = alloc_post(lx, ly);
    calc_post_flat(&fwd, &bwd, lx, ly, &mut post);
    post
}
